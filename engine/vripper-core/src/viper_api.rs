//! Viper lookup API client matching the reference thread/post parser contract.

use crate::api::{parse_thread_response, ParsedThread};
use crate::hosts::CookieMutation;
use crate::transport::{HttpTransport, RequestContext, TransportError};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::time::{sleep, Instant};
use url::Url;

const DEFAULT_LOOKUP_ENDPOINT: &str = "https://viper.click/vr.php";
const DEFAULT_RETRY_DELAY: Duration = Duration::from_secs(2);

#[derive(Debug, Error)]
pub enum ViperApiError {
    #[error("invalid lookup endpoint: {0}")]
    InvalidEndpoint(String),
    #[error(transparent)]
    Transport(#[from] TransportError),
    #[error("thread/post response parse failed: {0}")]
    Parse(String),
}

#[derive(Debug)]
struct BucketState {
    tokens: f64,
    updated: Instant,
}

#[derive(Debug)]
struct RequestLimiter {
    capacity: f64,
    refill_per_second: f64,
    state: Mutex<BucketState>,
}

impl RequestLimiter {
    fn new(request_limit_per_second: u8) -> Self {
        let capacity = f64::from(request_limit_per_second.clamp(1, 5));
        Self {
            capacity,
            refill_per_second: capacity,
            state: Mutex::new(BucketState {
                tokens: capacity,
                updated: Instant::now(),
            }),
        }
    }

    async fn acquire(&self) {
        loop {
            let wait = {
                let mut state = self.state.lock().await;
                let now = Instant::now();
                let elapsed = now.duration_since(state.updated).as_secs_f64();
                state.tokens = (state.tokens + elapsed * self.refill_per_second).min(self.capacity);
                state.updated = now;
                if state.tokens >= 1.0 {
                    state.tokens -= 1.0;
                    None
                } else {
                    Some(Duration::from_secs_f64(
                        (1.0 - state.tokens) / self.refill_per_second,
                    ))
                }
            };
            if let Some(duration) = wait {
                sleep(duration).await;
            } else {
                return;
            }
        }
    }
}

pub struct ViperApi {
    transport: HttpTransport,
    endpoint: Url,
    site_proxy: String,
    click_context: RequestContext,
    limiter: Arc<RequestLimiter>,
    max_attempts: u8,
}

impl ViperApi {
    pub fn new(
        transport: HttpTransport,
        site_proxy: impl Into<String>,
        request_limit_per_second: u8,
        max_attempts: u8,
        click_cookies: Vec<CookieMutation>,
    ) -> Result<Self, ViperApiError> {
        Self::with_endpoint(
            transport,
            DEFAULT_LOOKUP_ENDPOINT,
            site_proxy,
            request_limit_per_second,
            max_attempts,
            click_cookies,
        )
    }

    pub fn with_endpoint(
        transport: HttpTransport,
        endpoint: &str,
        site_proxy: impl Into<String>,
        request_limit_per_second: u8,
        max_attempts: u8,
        click_cookies: Vec<CookieMutation>,
    ) -> Result<Self, ViperApiError> {
        let endpoint =
            Url::parse(endpoint).map_err(|error| ViperApiError::InvalidEndpoint(error.to_string()))?;
        Ok(Self {
            transport,
            endpoint,
            site_proxy: site_proxy.into(),
            click_context: RequestContext {
                referer: None,
                cookies: click_cookies,
            },
            limiter: Arc::new(RequestLimiter::new(request_limit_per_second)),
            max_attempts: max_attempts.clamp(1, 10),
        })
    }

    pub async fn fetch_thread(&self, thread_id: i64) -> Result<ParsedThread, ViperApiError> {
        self.fetch_lookup("t", thread_id).await
    }

    pub async fn fetch_post(
        &self,
        _thread_id: i64,
        post_id: i64,
    ) -> Result<ParsedThread, ViperApiError> {
        self.fetch_lookup("p", post_id).await
    }

    async fn fetch_lookup(
        &self,
        parameter: &str,
        id: i64,
    ) -> Result<ParsedThread, ViperApiError> {
        let url = lookup_url(&self.endpoint, parameter, id);
        let mut last_error = None;

        for attempt in 1..=self.max_attempts {
            self.limiter.acquire().await;
            match self.transport.get_text(url.as_str(), &self.click_context).await {
                Ok(xml) => match parse_thread_response(&xml, &self.site_proxy) {
                    Ok(thread) => return Ok(thread),
                    Err(error) => last_error = Some(ViperApiError::Parse(error)),
                },
                Err(error) => last_error = Some(ViperApiError::Transport(error)),
            }

            if attempt < self.max_attempts {
                sleep(DEFAULT_RETRY_DELAY).await;
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ViperApiError::Parse("lookup failed without a response".into())
        }))
    }
}

pub fn lookup_url(endpoint: &Url, parameter: &str, id: i64) -> Url {
    let mut url = endpoint.clone();
    url.query_pairs_mut()
        .append_pair(parameter, &id.to_string());
    url
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::HttpTransport;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn builds_reference_thread_and_post_lookup_urls() {
        let endpoint = Url::parse(DEFAULT_LOOKUP_ENDPOINT).unwrap();
        assert_eq!(
            lookup_url(&endpoint, "t", 123).as_str(),
            "https://viper.click/vr.php?t=123"
        );
        assert_eq!(
            lookup_url(&endpoint, "p", 456).as_str(),
            "https://viper.click/vr.php?p=456"
        );
    }

    #[tokio::test]
    async fn fetches_thread_response_through_shared_transport() {
        let server = MockServer::start().await;
        let xml = std::fs::read_to_string("../../fixtures/api/thread-response.xml").unwrap();
        Mock::given(method("GET"))
            .and(path("/vr.php"))
            .and(query_param("t", "10001"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "application/xml")
                    .set_body_string(xml),
            )
            .mount(&server)
            .await;

        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let transport = HttpTransport::from_client(client);
        let api = ViperApi::with_endpoint(
            transport,
            &format!("{}/vr.php", server.uri()),
            "https://vipergirls.to",
            5,
            3,
            Vec::new(),
        )
        .unwrap();

        let thread = api.fetch_thread(10001).await.unwrap();
        assert_eq!(thread.thread_id, 10001);
        assert_eq!(thread.posts.len(), 2);
        assert_eq!(thread.posts[0].images.len(), 2);
    }

    #[tokio::test]
    async fn applies_click_auth_cookies_to_lookup_requests() {
        let server = MockServer::start().await;
        let xml = r#"<thread id="1" title="T"></thread>"#;
        Mock::given(method("GET"))
            .and(path("/vr.php"))
            .and(query_param("p", "7"))
            .and(wiremock::matchers::header("cookie", "vg_userid=123; vg_password=secret-cookie"))
            .respond_with(ResponseTemplate::new(200).set_body_string(xml))
            .mount(&server)
            .await;

        let test_host = Url::parse(&server.uri()).unwrap().host_str().unwrap().to_owned();
        let cookies = vec![
            CookieMutation {
                name: "vg_userid".into(),
                value: "123".into(),
                domain: test_host.clone(),
                path: "/".into(),
                max_age_seconds: None,
            },
            CookieMutation {
                name: "vg_password".into(),
                value: "secret-cookie".into(),
                domain: test_host,
                path: "/".into(),
                max_age_seconds: None,
            },
        ];
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let api = ViperApi::with_endpoint(
            HttpTransport::from_client(client),
            &format!("{}/vr.php", server.uri()),
            "https://vipergirls.to",
            5,
            1,
            cookies,
        )
        .unwrap();

        let thread = api.fetch_post(1, 7).await.unwrap();
        assert_eq!(thread.thread_id, 1);
    }
}
