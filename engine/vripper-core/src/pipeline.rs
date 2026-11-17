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
use std::path::{Path, PathBuf};
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
        })
    }

    pub fn from_transport(
        transport: HttpTransport,
        temp_directory: impl Into<PathBuf>,
        fetch_original_imx_filename: bool,
    ) -> Self {
        Self {
            transport,
            temp_directory: temp_directory.into(),
            fetch_original_imx_filename,
        }
    }

    /// Resolve the host's final image URL, stream the image to a VRipper-style
    /// temporary file, and move it to the exact requested destination filename.
    pub async fn download_image(
        &self,
        host: HostId,
        source_url: &str,
        thumbnail_url: &str,
        image_index_zero_based: usize,
        force_order: bool,
        destination_directory: &Path,
    ) -> Result<CompletedImage, PipelineError> {
        let resolved = self.resolve_image(host, source_url, thumbnail_url).await?;
        let context = RequestContext {
            referer: resolved
                .referer_override
                .clone()
                .or_else(|| Some(DEFAULT_REFERER.to_owned())),
            cookies: resolved.cookies.clone(),
        };
        let temporary = self
            .transport
            .download_to_temp(&resolved.image_url, &context, &self.temp_directory)
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
            cookies: Vec::new(),
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
    use crate::ImageMime;

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
