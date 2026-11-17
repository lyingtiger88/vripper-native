//! Shared asynchronous HTTP transport used by every application network operation.
//! This module does not implement site-specific logic or bypass site access controls.

use crate::hosts::{CookieMutation, FollowUpMethod, FollowUpRequest};
use crate::network::{NetworkConfig, ProxyCredentials, ProxyEndpoint, ProxyError, build_http_client_with_timeout};
use crate::ImageMime;
use futures_util::StreamExt;
use reqwest::header::{CONTENT_TYPE, COOKIE, REFERER, SET_COOKIE};
use reqwest::{Client, RequestBuilder, Response};
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use url::Url;

#[derive(Debug, Clone, Default)]
pub struct RequestContext {
    pub referer: Option<String>,
    pub cookies: Vec<CookieMutation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormPostResponse {
    pub body: String,
    pub cookies: Vec<CookieMutation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadMetadata {
    pub status: u16,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadedTempFile {
    pub path: PathBuf,
    pub mime: ImageMime,
    pub bytes_written: u64,
    pub expected_bytes: Option<u64>,
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error(transparent)]
    Proxy(#[from] ProxyError),
    #[error("HTTP request failed: {0}")]
    Request(String),
    #[error("server returned HTTP status {0}")]
    HttpStatus(u16),
    #[error("unsupported or missing image MIME type: {0}")]
    UnsupportedMime(String),
    #[error("invalid HTTP header: {0}")]
    InvalidHeader(String),
    #[error("file operation failed: {0}")]
    File(String),
}

#[derive(Clone)]
pub struct HttpTransport {
    client: Client,
}

impl HttpTransport {
    pub fn new(
        config: &NetworkConfig,
        credentials: Option<&ProxyCredentials>,
        system_proxy: Option<&ProxyEndpoint>,
    ) -> Result<Self, TransportError> {
        Self::new_with_timeout(
            config,
            credentials,
            system_proxy,
            Duration::from_secs(30),
        )
    }

    pub fn new_with_timeout(
        config: &NetworkConfig,
        credentials: Option<&ProxyCredentials>,
        system_proxy: Option<&ProxyEndpoint>,
        timeout: Duration,
    ) -> Result<Self, TransportError> {
        Ok(Self {
            client: build_http_client_with_timeout(config, credentials, system_proxy, timeout)?,
        })
    }

    pub fn from_client(client: Client) -> Self {
        Self { client }
    }

    pub async fn head(
        &self,
        url: &str,
        context: &RequestContext,
    ) -> Result<HeadMetadata, TransportError> {
        let response = self.apply_context(self.client.head(url), url, context)
            .send()
            .await
            .map_err(|error| TransportError::Request(error.to_string()))?;
        Ok(HeadMetadata {
            status: response.status().as_u16(),
            content_type: header_string(&response, CONTENT_TYPE),
            content_length: response.content_length(),
        })
    }

    pub async fn get_text(
        &self,
        url: &str,
        context: &RequestContext,
    ) -> Result<String, TransportError> {
        let response = self.apply_context(self.client.get(url), url, context)
            .send()
            .await
            .map_err(|error| TransportError::Request(error.to_string()))?;
        let response = checked_response(response)?;
        response
            .text()
            .await
            .map_err(|error| TransportError::Request(error.to_string()))
    }

    pub async fn execute_follow_up(
        &self,
        request: &FollowUpRequest,
    ) -> Result<String, TransportError> {
        let builder = match request.method {
            FollowUpMethod::Get => self.client.get(&request.url),
            FollowUpMethod::Post => self.client.post(&request.url),
        };
        let context = RequestContext {
            referer: request.referer.clone(),
            cookies: request.cookies.clone(),
        };
        let mut builder = self.apply_context(builder, &request.url, &context);
        if request.method == FollowUpMethod::Post && !request.form_fields.is_empty() {
            let fields = request
                .form_fields
                .iter()
                .map(|(key, value)| (key.as_str(), value.as_str()))
                .collect::<Vec<_>>();
            builder = builder.form(&fields);
        }
        let response = builder
            .send()
            .await
            .map_err(|error| TransportError::Request(error.to_string()))?;
        checked_response(response)?
            .text()
            .await
            .map_err(|error| TransportError::Request(error.to_string()))
    }


    pub async fn post_form(
        &self,
        url: &str,
        fields: &[(String, String)],
        context: &RequestContext,
    ) -> Result<FormPostResponse, TransportError> {
        let builder = self.client.post(url);
        let builder = self.apply_context(builder, url, context).form(fields);
        let response = builder
            .send()
            .await
            .map_err(|error| TransportError::Request(error.to_string()))?;
        let response = checked_response(response)?;
        let cookies = response
            .headers()
            .get_all(SET_COOKIE)
            .iter()
            .filter_map(|header| header.to_str().ok())
            .filter_map(|header| parse_set_cookie(header, url))
            .collect();
        let body = response
            .text()
            .await
            .map_err(|error| TransportError::Request(error.to_string()))?;
        Ok(FormPostResponse { body, cookies })
    }

    pub async fn download_to_temp(
        &self,
        url: &str,
        context: &RequestContext,
        temp_directory: &Path,
    ) -> Result<DownloadedTempFile, TransportError> {
        tokio::fs::create_dir_all(temp_directory)
            .await
            .map_err(|error| TransportError::File(error.to_string()))?;

        let response = self.apply_context(self.client.get(url), url, context)
            .send()
            .await
            .map_err(|error| TransportError::Request(error.to_string()))?;
        let response = checked_response(response)?;
        let raw_mime = header_string(&response, CONTENT_TYPE).unwrap_or_default();
        let mime_name = raw_mime.split(';').next().unwrap_or_default().trim().to_ascii_lowercase();
        let mime = ImageMime::from_content_type(&mime_name)
            .ok_or_else(|| TransportError::UnsupportedMime(raw_mime.clone()))?;
        let expected_bytes = response.content_length();

        let temporary = tempfile::Builder::new()
            .prefix("vripper_")
            .suffix(".tmp")
            .tempfile_in(temp_directory)
            .map_err(|error| TransportError::File(error.to_string()))?;
        let mut file = tokio::fs::File::from_std(
            temporary
                .reopen()
                .map_err(|error| TransportError::File(error.to_string()))?,
        );
        let mut stream = response.bytes_stream();
        let mut bytes_written = 0_u64;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|error| TransportError::Request(error.to_string()))?;
            file.write_all(&chunk)
                .await
                .map_err(|error| TransportError::File(error.to_string()))?;
            bytes_written += chunk.len() as u64;
        }
        file.flush()
            .await
            .map_err(|error| TransportError::File(error.to_string()))?;
        drop(file);

        let path = temporary
            .into_temp_path()
            .keep()
            .map_err(|error| TransportError::File(error.to_string()))?;

        Ok(DownloadedTempFile {
            path,
            mime,
            bytes_written,
            expected_bytes,
        })
    }

    fn apply_context(
        &self,
        mut builder: RequestBuilder,
        request_url: &str,
        context: &RequestContext,
    ) -> RequestBuilder {
        if let Some(referer) = &context.referer {
            builder = builder.header(REFERER, referer);
        }
        if let Some(cookie_header) = cookie_header_for_url(request_url, &context.cookies) {
            builder = builder.header(COOKIE, cookie_header);
        }
        builder
    }
}

fn checked_response(response: Response) -> Result<Response, TransportError> {
    if !response.status().is_success() {
        return Err(TransportError::HttpStatus(response.status().as_u16()));
    }
    Ok(response)
}

fn header_string(response: &Response, name: reqwest::header::HeaderName) -> Option<String> {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}


fn parse_set_cookie(header: &str, request_url: &str) -> Option<CookieMutation> {
    let parsed_url = Url::parse(request_url).ok()?;
    let mut parts = header.split(';').map(str::trim);
    let (name, value) = parts.next()?.split_once('=')?;
    if name.trim().is_empty() {
        return None;
    }

    let mut domain = parsed_url.host_str()?.to_owned();
    let mut path = "/".to_owned();
    let mut max_age_seconds = None;
    for part in parts {
        if let Some((key, value)) = part.split_once('=') {
            match key.trim().to_ascii_lowercase().as_str() {
                "domain" => domain = value.trim().to_ascii_lowercase(),
                "path" if value.trim().starts_with('/') => path = value.trim().to_owned(),
                "max-age" => max_age_seconds = value.trim().parse::<u64>().ok(),
                _ => {}
            }
        }
    }

    Some(CookieMutation {
        name: name.trim().to_owned(),
        value: value.trim().to_owned(),
        domain,
        path,
        max_age_seconds,
    })
}

fn cookie_header_for_url(url: &str, cookies: &[CookieMutation]) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    let host = parsed.host_str()?;
    let path = parsed.path();
    let values = cookies
        .iter()
        .filter(|cookie| {
            let domain = cookie.domain.trim_start_matches('.');
            let path_matches = path == cookie.path
                || path.starts_with(&format!("{}/", cookie.path.trim_end_matches('/')));
            (host == domain || host.ends_with(&format!(".{domain}"))) && path_matches
        })
        .map(|cookie| format!("{}={}", cookie.name, cookie.value))
        .collect::<Vec<_>>();
    if values.is_empty() {
        None
    } else {
        Some(values.join("; "))
    }
}

impl ImageMime {
    pub fn from_content_type(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "image/bmp" => Some(Self::Bmp),
            "image/gif" => Some(Self::Gif),
            "image/jpeg" | "image/jpg" => Some(Self::Jpeg),
            "image/png" => Some(Self::Png),
            "image/webp" => Some(Self::Webp),
            "image/svg+xml" => Some(Self::Svg),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_only_reference_supported_image_mime_types() {
        assert_eq!(ImageMime::from_content_type("image/bmp"), Some(ImageMime::Bmp));
        assert_eq!(ImageMime::from_content_type("image/gif"), Some(ImageMime::Gif));
        assert_eq!(ImageMime::from_content_type("image/jpeg"), Some(ImageMime::Jpeg));
        assert_eq!(ImageMime::from_content_type("image/png"), Some(ImageMime::Png));
        assert_eq!(ImageMime::from_content_type("image/webp"), Some(ImageMime::Webp));
        assert_eq!(ImageMime::from_content_type("image/svg+xml"), Some(ImageMime::Svg));
        assert_eq!(ImageMime::from_content_type("application/octet-stream"), None);
    }

    #[test]
    fn cookie_header_respects_domain_and_path_scope() {
        let cookies = vec![
            CookieMutation {
                name: "nsfw_inter".into(),
                value: "1".into(),
                domain: "www.imagebam.com".into(),
                path: "/".into(),
                max_age_seconds: Some(86400),
            },
            CookieMutation {
                name: "other".into(),
                value: "2".into(),
                domain: "unrelated.example".into(),
                path: "/".into(),
                max_age_seconds: None,
            },
        ];
        assert_eq!(
            cookie_header_for_url("https://www.imagebam.com/view/1", &cookies).as_deref(),
            Some("nsfw_inter=1")
        );
        assert_eq!(
            cookie_header_for_url("https://imagebam.com/view/1", &cookies),
            None
        );
    }

    #[test]
    fn cookie_header_respects_cookie_path() {
        let cookies = vec![CookieMutation {
            name: "session".into(),
            value: "secret".into(),
            domain: "example.test".into(),
            path: "/account".into(),
            max_age_seconds: None,
        }];
        assert_eq!(
            cookie_header_for_url("https://example.test/account/profile", &cookies).as_deref(),
            Some("session=secret")
        );
        assert_eq!(
            cookie_header_for_url("https://example.test/images/a.jpg", &cookies),
            None
        );
    }
}
