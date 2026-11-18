//! End-to-end coordinator from pasted links to queued image downloads.

use crate::api::{ParsedHostCount, ParsedImage, ParsedThread};
use crate::hosts::CookieMutation;
use crate::network::{NetworkConfig, ProxyCredentials, ProxyEndpoint};
use crate::pipeline::HostPipeline;
use crate::queue::{QueueError, QueueJob, QueueManager, QueueMove, QueueSettings, QueueSnapshot};
use crate::scan_links;
use crate::transport::{HttpTransport, TransportError};
use crate::viper_api::{ViperApi, ViperApiError};
use crate::viper_auth::{ViperAuthError, ViperAuthService, ViperCredentials};
use crate::{post_folder_name, safe_calculate_download_directory, validate_download_component};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use thiserror::Error;
use tokio::sync::broadcast;

#[derive(Clone, Deserialize)]
pub struct CoordinatorConfig {
    pub network: NetworkConfig,
    pub proxy_credentials: Option<ProxyCredentials>,
    pub system_proxy: Option<ProxyEndpoint>,
    pub viper_host: String,
    pub site_proxy: String,
    pub viper_credentials: Option<ViperCredentials>,
    pub request_limit_per_second: u8,
    pub max_attempts: u8,
    pub connection_timeout_secs: u64,
    pub queue_settings: QueueSettings,
    pub download_path: PathBuf,
    pub temp_path: PathBuf,
    #[serde(default)]
    pub queue_database_path: Option<PathBuf>,
    #[serde(default)]
    pub legacy_database_path: Option<PathBuf>,
    pub forum_subdirectory: bool,
    pub thread_sublocation: bool,
    pub append_post_id: bool,
    pub force_order: bool,
    pub fetch_original_imx_filename: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct ThreadRecord {
    pub thread_id: i64,
    pub title: String,
    pub forum: String,
    pub url: String,
    pub post_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct PostRecord {
    pub thread_id: i64,
    pub thread_title: String,
    pub post_id: i64,
    pub post_number: i32,
    pub title: String,
    pub image_count: usize,
    pub url: String,
    pub forum: String,
    pub hosts: Vec<ParsedHostCount>,
    pub images: Vec<ParsedImage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct CoordinatorSummary {
    pub thread_lookups: usize,
    pub post_lookups: usize,
    pub posts_discovered: usize,
    pub images_queued: usize,
    pub threads: Vec<ThreadRecord>,
    pub posts: Vec<PostRecord>,
    pub invalid_links: Vec<String>,
    pub errors: Vec<String>,
    pub queue_job_ids: Vec<u64>,
}


#[derive(Debug, Error)]
pub enum CoordinatorError {
    #[error(transparent)]
    Transport(#[from] TransportError),
    #[error(transparent)]
    Authentication(#[from] ViperAuthError),
    #[error(transparent)]
    Lookup(#[from] ViperApiError),
    #[error(transparent)]
    Queue(#[from] QueueError),
    #[error(transparent)]
    Database(#[from] crate::database::DatabaseError),
}

struct DownloadOptions {
    download_path: PathBuf,
    forum_subdirectory: bool,
    thread_sublocation: bool,
    append_post_id: bool,
    force_order: bool,
    site_proxy: String,
}

pub struct DownloadCoordinator {
    api: ViperApi,
    queue: QueueManager,
    options: DownloadOptions,
    legacy_database_path: Option<PathBuf>,
}

impl DownloadCoordinator {
    pub async fn new(config: CoordinatorConfig) -> Result<Self, CoordinatorError> {
        let options = DownloadOptions {
            download_path: config.download_path.clone(),
            forum_subdirectory: config.forum_subdirectory,
            thread_sublocation: config.thread_sublocation,
            append_post_id: config.append_post_id,
            force_order: config.force_order,
            site_proxy: config.site_proxy.clone(),
        };

        let transport = HttpTransport::new_with_timeout(
            &config.network,
            config.proxy_credentials.as_ref(),
            config.system_proxy.as_ref(),
            std::time::Duration::from_secs(config.connection_timeout_secs.clamp(1, 300)),
        )?;

        let (click_cookies, site_cookies): (Vec<CookieMutation>, Vec<CookieMutation>) =
            if let Some(credentials) = &config.viper_credentials {
                let session = ViperAuthService::new(transport.clone())
                    .authenticate(&config.viper_host, credentials)
                    .await?;
                (session.click_cookies, session.site_cookies)
            } else {
                (Vec::new(), Vec::new())
            };

        let api = ViperApi::new(
            transport.clone(),
            config.site_proxy.clone(),
            config.request_limit_per_second,
            config.max_attempts,
            click_cookies,
        )?;

        let legacy_database_path = config.legacy_database_path.clone();

        let pipeline = HostPipeline::from_transport_with_options(
            transport,
            config.temp_path.clone(),
            config.fetch_original_imx_filename,
            site_cookies,
            config.max_attempts,
        );
        let queue = match config.queue_database_path {
            Some(path) => QueueManager::persistent(pipeline, config.queue_settings, &path)?,
            None => QueueManager::new(pipeline, config.queue_settings),
        };

        if let Some(path) = legacy_database_path.clone() {
            let result = async {
                let entries = crate::database::read_legacy_queue(&path)?;
                queue.import_legacy(entries).await?;
                Ok::<(), CoordinatorError>(())
            }.await;
            if let Err(error) = result {
                let _ = queue.shutdown().await;
                return Err(error);
            }
        }
        Ok(Self { api, queue, options, legacy_database_path })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<crate::queue::QueueEvent> {
        self.queue.subscribe()
    }

    pub async fn snapshot(&self) -> Result<QueueSnapshot, CoordinatorError> {
        Ok(self.queue.snapshot().await?)
    }

    pub async fn retry(&self, id: u64) -> Result<bool, CoordinatorError> {
        Ok(self.queue.retry(id).await?)
    }

    pub async fn stop(&self, id: u64) -> Result<bool, CoordinatorError> {
        Ok(self.queue.stop(id).await?)
    }

    pub async fn stop_all(&self) -> Result<(), CoordinatorError> {
        Ok(self.queue.stop_all().await?)
    }

    pub async fn move_job(
        &self,
        id: u64,
        movement: QueueMove,
    ) -> Result<bool, CoordinatorError> {
        Ok(self.queue.move_job(id, movement).await?)
    }

    pub async fn clear_finished(&self) -> Result<(), CoordinatorError> {
        Ok(self.queue.clear_finished().await?)
    }

    pub async fn shutdown(&self) -> Result<(), CoordinatorError> {
        Ok(self.queue.shutdown().await?)
    }

    /// Scan multiline input, perform thread/post lookups, deduplicate returned
    /// posts and enqueue each linked image using the reference output path rules.
    fn legacy_database_path(&self) -> Option<PathBuf> {
        self.legacy_database_path.clone()
    }

    pub async fn scan_and_queue(
        &self,
        input: &str,
        configured_bases: &[String],
    ) -> CoordinatorSummary {
        let scan = scan_links(input, configured_bases);
        let mut summary = CoordinatorSummary {
            invalid_links: scan.invalid,
            ..CoordinatorSummary::default()
        };
        let mut parsed_threads = Vec::<ParsedThread>::new();

        for thread_link in scan.threads {
            summary.thread_lookups += 1;
            match i64::try_from(thread_link.thread_id) {
                Ok(thread_id) => match self.api.fetch_thread(thread_id).await {
                    Ok(thread) => parsed_threads.push(thread),
                    Err(error) => summary.errors.push(error.to_string()),
                },
                Err(_) => summary.errors.push("thread ID exceeds supported integer range".into()),
            }
        }

        for post_link in scan.posts {
            summary.post_lookups += 1;
            match (i64::try_from(post_link.thread_id), i64::try_from(post_link.post_id)) {
                (Ok(thread_id), Ok(post_id)) => match self.api.fetch_post(thread_id, post_id).await {
                    Ok(thread) => parsed_threads.push(thread),
                    Err(error) => summary.errors.push(error.to_string()),
                },
                _ => summary.errors.push("post or thread ID exceeds supported integer range".into()),
            }
        }

        let mut seen_posts = HashSet::<(i64, i64)>::new();
        let mut seen_threads = HashSet::<i64>::new();
        for thread in parsed_threads {
            if !thread.error.is_empty() {
                summary.errors.push(thread.error.clone());
            }
            let thread_url = format!("{}/threads/{}", self.options.site_proxy.trim_end_matches('/'), thread.thread_id);
            if seen_threads.insert(thread.thread_id) {
                summary.threads.push(ThreadRecord {
                    thread_id: thread.thread_id,
                    title: thread.title.clone(),
                    forum: thread.forum.clone(),
                    url: thread_url,
                    post_count: 0,
                });
            }
            for post in thread.posts.iter().cloned() {
                if !seen_posts.insert((post.thread_id, post.post_id)) {
                    continue;
                }
                summary.posts_discovered += 1;
                let record = PostRecord {
                    thread_id: post.thread_id,
                    thread_title: post.thread_title.clone(),
                    post_id: post.post_id,
                    post_number: post.post_number,
                    title: post.title.clone(),
                    image_count: post.image_count,
                    url: post.url.clone(),
                    forum: post.forum.clone(),
                    hosts: post.hosts.clone(),
                    images: post.images.clone(),
                };
                summary.posts.push(record);
                if let Some(thread_record) = summary.threads.iter_mut().find(|item| item.thread_id == post.thread_id) {
                    thread_record.post_count += 1;
                }

                let base_directory = match safe_calculate_download_directory(
                    &self.options.download_path,
                    &post.forum,
                    &post.thread_title,
                    self.options.forum_subdirectory,
                    self.options.thread_sublocation,
                ) {
                    Ok(path) => path,
                    Err(error) => {
                        summary.errors.push(format!(
                            "unsafe output directory for post {}: {error}",
                            post.post_id
                        ));
                        continue;
                    }
                };
                let folder = post_folder_name(
                    &post.title,
                    post.post_id.max(0) as u64,
                    self.options.append_post_id,
                );
                if let Err(error) = validate_download_component(&folder) {
                    summary.errors.push(format!(
                        "unsafe output folder for post {}: {error}",
                        post.post_id
                    ));
                    continue;
                }
                let destination_directory = base_directory.join(folder.clone());

                if let Some(database_path) = self.legacy_database_path() {
                    if let Err(error) = crate::database::persist_extracted_post(
                        &database_path,
                        &thread,
                        &post,
                        &base_directory,
                        &folder,
                        &self.options.site_proxy,
                    ) {
                        summary.errors.push(format!("database persistence failed for post {}: {error}", post.post_id));
                    }
                }

                for (index, image) in post.images.into_iter().enumerate() {
                    let job = QueueJob {
                        id: 0,
                        host: image.host,
                        source_url: image.main_url,
                        thumbnail_url: image.thumbnail_url,
                        image_index_zero_based: index,
                        force_order: self.options.force_order,
                        destination_directory: destination_directory.clone(),
                    };
                    match self.queue.add(job).await {
                        Ok(id) => {
                            summary.images_queued += 1;
                            summary.queue_job_ids.push(id);
                        }
                        Err(error) => summary.errors.push(error.to_string()),
                    }
                }
            }
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_starts_empty() {
        let summary = CoordinatorSummary::default();
        assert_eq!(summary.thread_lookups, 0);
        assert_eq!(summary.images_queued, 0);
        assert!(summary.invalid_links.is_empty());
    }
}
