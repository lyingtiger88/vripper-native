//! Host-resolution and image-download orchestration.
//!
//! This is the first end-to-end local engine path. Every request goes through the
//! same HttpTransport instance so the selected proxy and TLS policy apply uniformly.

use crate::final_image_filename;
use crate::hosts::{
    default_image_name, resolve_host_html, HostId, HostResolution,
    ResolvedImage,
};
use crate::network::{NetworkConfig, ProxyCredentials, ProxyEndpoint};
use crate::transport::{DownloadedTempFile, HttpTransport, RequestContext, TransportError};
use crate::ImageMime;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::time::sleep;
use thiserror::Error;

const DEFAULT_REFERER: &str = "https://vipergirls.to/";

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error(transparent)]
    Transport(#[from] TransportError),
    #[error("host resolver failed: {0}")]
    HostResolution(String),
    #[error("file finalization failed: {0}")]
    File(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedImage {
    pub path: PathBuf,
    pub filename: String,
    pub mime: ImageMime,
    pub bytes_written: u64,
    pub expected_bytes: Option<u64>,
    pub complete: bool,
}

#[derive(Clone)]
pub struct HostPipeline {
    transport: HttpTransport,
    temp_directory: PathBuf,
    fetch_original_imx_filename: bool,
    site_cookies: Vec<crate::hosts::CookieMutation>,
    max_attempts: u8,
}

impl HostPipeline {
    pub fn new(
        network: &NetworkConfig,
        credentials: Option<&ProxyCredentials>,
        system_proxy: Option<&ProxyEndpoint>,
        temp_directory: impl Into<PathBuf>,
        fetch_original_imx_filename: bool,
    ) -> Result<Self, PipelineError> {
        Ok(Self {
            transport: HttpTransport::new(network, credentials, system_proxy)?,
            temp_directory: temp_directory.into(),
            fetch_original_imx_filename,
            site_cookies: Vec::new(),
            max_attempts: 3,
        })
    }

    pub fn from_transport(
        transport: HttpTransport,
        temp_directory: impl Into<PathBuf>,
        fetch_original_imx_filename: bool,
    ) -> Self {
        Self::from_transport_with_cookies(
            transport,
            temp_directory,
            fetch_original_imx_filename,
            Vec::new(),
        )
    }

    pub fn from_transport_with_cookies(
        transport: HttpTransport,
        temp_directory: impl Into<PathBuf>,
        fetch_original_imx_filename: bool,
        site_cookies: Vec<crate::hosts::CookieMutation>,
    ) -> Self {
        Self::from_transport_with_options(
            transport,
            temp_directory,
            fetch_original_imx_filename,
            site_cookies,
            3,
        )
    }

    pub fn from_transport_with_options(
        transport: HttpTransport,
        temp_directory: impl Into<PathBuf>,
        fetch_original_imx_filename: bool,
        site_cookies: Vec<crate::hosts::CookieMutation>,
        max_attempts: u8,
    ) -> Self {
        Self {
            transport,
            temp_directory: temp_directory.into(),
            fetch_original_imx_filename,
            site_cookies,
            max_attempts: max_attempts.clamp(1, 10),
        }
    }

    /// Resolve the host's final image URL, stream the image to a VRipper-style
    /// temporary file, and move it to the exact requested destination filename.
    pub async fn download_image<F>(
        &self,
        host: HostId,
        source_url: &str,
        thumbnail_url: &str,
        image_index_zero_based: usize,
        force_order: bool,
        destination_directory: &Path,
        on_progress: F,
    ) -> Result<CompletedImage, PipelineError>
    where
        F: Fn(u64, Option<u64>) + Send + Sync,
    {
        let mut last_error = None;
        for attempt in 1..=self.max_attempts {
            match self
                .download_image_once(
                    host,
                    source_url,
                    thumbnail_url,
                    image_index_zero_based,
                    force_order,
                    destination_directory,
                    &on_progress,
                )
                .await
            {
                Ok(result) => return Ok(result),
                Err(error) => last_error = Some(error),
            }
            if attempt < self.max_attempts {
                sleep(Duration::from_secs(2)).await;
            }
        }
        Err(last_error.unwrap_or_else(|| {
            PipelineError::HostResolution("download failed without an attempt".into())
        }))
    }

    async fn download_image_once(
        &self,
        host: HostId,
        source_url: &str,
        thumbnail_url: &str,
        image_index_zero_based: usize,
        force_order: bool,
        destination_directory: &Path,
        on_progress: &impl Fn(u64, Option<u64>) + Send + Sync,
    ) -> Result<CompletedImage, PipelineError> {
        let resolved = self.resolve_image(host, source_url, thumbnail_url).await?;
        let context = RequestContext {
            referer: resolved
                .referer_override
                .clone()
                .or_else(|| Some(DEFAULT_REFERER.to_owned())),
            cookies: self
                .site_cookies
                .iter()
                .cloned()
                .chain(resolved.cookies.iter().cloned())
                .collect(),
        };
        let temporary = self
            .transport
            .download_to_temp(&resolved.image_url, &context, &self.temp_directory, |bytes, expected| on_progress(bytes, expected))
            .await?;

        self.finalize(
            temporary,
            &resolved.name,
            image_index_zero_based,
            force_order,
            destination_directory,
        )
        .await
    }

    async fn resolve_image(
        &self,
        host: HostId,
        source_url: &str,
        thumbnail_url: &str,
    ) -> Result<ResolvedImage, PipelineError> {
        let default_context = RequestContext {
            referer: Some(DEFAULT_REFERER.to_owned()),
            cookies: self.site_cookies.clone(),
        };

        // Imx host ID 8 is a special case in the reference base Host implementation:
        // it bypasses the generic HEAD/direct-image branch.
        if host == HostId::Imx {
            let initial_html = if self.fetch_original_imx_filename {
                let page_url = crate::hosts::imx_page_url(source_url);
                self.transport.get_text(&page_url, &default_context).await?
            } else {
                String::new()
            };
            return self
                .resolve_with_follow_up(
                    host,
                    source_url,
                    thumbnail_url,
                    &initial_html,
                )
                .await;
        }

        let head = self.transport.head(source_url, &default_context).await?;
        if head
            .content_type
            .as_deref()
            .map(|content_type| content_type.to_ascii_lowercase().starts_with("image/"))
            .unwrap_or(false)
        {
            return Ok(ResolvedImage {
                name: default_image_name(source_url),
                image_url: source_url.to_owned(),
                referer_override: None,
                cookies: Vec::new(),
            });
        }

        // Pimpandhost's resolver explicitly requests its size=original URL. The
        // initial page body is not consumed by the reference resolver.
        let initial_html = if host == HostId::Pimpandhost {
            String::new()
        } else {
            self.transport.get_text(source_url, &default_context).await?
        };

        self.resolve_with_follow_up(host, source_url, thumbnail_url, &initial_html)
            .await
    }

    async fn resolve_with_follow_up(
        &self,
        host: HostId,
        source_url: &str,
        thumbnail_url: &str,
        initial_html: &str,
    ) -> Result<ResolvedImage, PipelineError> {
        let resolution = resolve_host_html(
            host,
            source_url,
            thumbnail_url,
            initial_html,
            None,
            self.fetch_original_imx_filename,
        )
        .map_err(PipelineError::HostResolution)?;

        match resolution {
            HostResolution::Resolved(image) => Ok(image),
            HostResolution::FollowUp(request) => {
                let mut follow_up_request = request;
                if follow_up_request.referer.is_none() {
                    follow_up_request.referer = Some(DEFAULT_REFERER.to_owned());
                }
                follow_up_request.cookies.extend(self.site_cookies.iter().cloned());
                let follow_up_html = self.transport.execute_follow_up(&follow_up_request).await?;
                let second = resolve_host_html(
                    host,
                    source_url,
                    thumbnail_url,
                    initial_html,
                    Some(&follow_up_html),
                    self.fetch_original_imx_filename,
                )
                .map_err(PipelineError::HostResolution)?;
                match second {
                    HostResolution::Resolved(image) => Ok(image),
                    HostResolution::FollowUp(next) => {
                        Err(PipelineError::HostResolution(format!(
                            "unexpected second continuation request: {:?} {}",
                            next.method, next.url
                        )))
                    }
                }
            }
        }
    }

    async fn finalize(
        &self,
        temporary: DownloadedTempFile,
        supplied_name: &str,
        image_index_zero_based: usize,
        force_order: bool,
        destination_directory: &Path,
    ) -> Result<CompletedImage, PipelineError> {
        tokio::fs::create_dir_all(destination_directory)
            .await
            .map_err(|error| PipelineError::File(error.to_string()))?;

        let filename = final_image_filename(
            supplied_name,
            temporary.mime,
            image_index_zero_based,
            force_order,
        );
        let destination = destination_directory.join(&filename);
        let copy_result = tokio::fs::copy(&temporary.path, &destination).await;
        let cleanup_result = tokio::fs::remove_file(&temporary.path).await;

        if let Err(error) = copy_result {
            let _ = cleanup_result;
            return Err(PipelineError::File(error.to_string()));
        }
        cleanup_result.map_err(|error| PipelineError::File(error.to_string()))?;

        let complete = temporary
            .expected_bytes
            .map(|expected| expected > 0 && expected == temporary.bytes_written)
            .unwrap_or(false);

        Ok(CompletedImage {
            path: destination,
            filename,
            mime: temporary.mime,
            bytes_written: temporary.bytes_written,
            expected_bytes: temporary.expected_bytes,
            complete,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[tokio::test]
    async fn downloads_direct_image_through_head_and_stream_pipeline() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let bytes = vec![0xFF, 0xD8, 0xFF, 0xD9];
        Mock::given(method("HEAD"))
            .and(path("/direct.jpg"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "image/jpeg")
                    .insert_header("content-length", "4"),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/direct.jpg"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "image/jpeg")
                    .set_body_bytes(bytes.clone()),
            )
            .mount(&server)
            .await;

        let temp = tempfile::tempdir().unwrap();
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let pipeline = HostPipeline::from_transport(
            crate::transport::HttpTransport::from_client(client),
            temp.path().join("temp"),
            false,
        );
        let source_url = format!("{}/direct.jpg", server.uri());
        let result = pipeline
            .download_image(
                HostId::DPicMe,
                &source_url,
                "",
                0,
                false,
                &temp.path().join("downloads"),
                |_, _| {},
            )
            .await
            .unwrap();

        assert_eq!(result.filename, "direct.JPG");
        assert_eq!(result.bytes_written, 4);
        assert!(result.complete);
        assert_eq!(tokio::fs::read(result.path).await.unwrap(), bytes);
    }

    #[tokio::test]
    async fn resolves_host_html_then_downloads_image_through_shared_transport() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let image_url = format!("{}/images/pic.jpg", server.uri());
        let page = format!(
            r#"<html><body><img id="pic" alt="fixture-picture" src="{image_url}"></body></html>"#
        );
        Mock::given(method("HEAD"))
            .and(path("/view/1"))
            .respond_with(
                ResponseTemplate::new(200).insert_header("content-type", "text/html"),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/view/1"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "text/html")
                    .set_body_string(page),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/images/pic.jpg"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "image/jpeg")
                    .insert_header("content-length", "4")
                    .set_body_bytes(vec![0xFF, 0xD8, 0xFF, 0xD9]),
            )
            .mount(&server)
            .await;

        let temp = tempfile::tempdir().unwrap();
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let pipeline = HostPipeline::from_transport(
            crate::transport::HttpTransport::from_client(client),
            temp.path().join("temp"),
            false,
        );
        let source_url = format!("{}/view/1", server.uri());
        let result = pipeline
            .download_image(
                HostId::DPicMe,
                &source_url,
                "",
                0,
                false,
                &temp.path().join("downloads"),
                |_, _| {},
            )
            .await
            .unwrap();

        assert_eq!(result.filename, "fixture-picture.JPG");
        assert_eq!(result.bytes_written, 4);
        assert!(result.complete);
        assert!(result.path.exists());
    }

    #[test]
    fn default_referer_matches_reference_host_base() {
        assert_eq!(DEFAULT_REFERER, "https://vipergirls.to/");
    }

    #[test]
    fn image_ordering_uses_reference_three_digit_prefix() {
        assert_eq!(
            final_image_filename("sample", ImageMime::Jpeg, 4, true),
            "005_sample.JPG"
        );
    }
}
