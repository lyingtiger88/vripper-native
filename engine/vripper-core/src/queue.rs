//! Bounded asynchronous download queue with per-host and global concurrency.

use crate::hosts::HostId;
use crate::pipeline::{CompletedImage, HostPipeline, PipelineError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::task::{AbortHandle, JoinSet};

static NEXT_JOB_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct QueueSettings {
    /// Zero means the reference fallback pool size of 24.
    pub max_global_concurrent: usize,
    /// Reference validation range is 1..=4.
    pub max_per_host_concurrent: usize,
}

impl Default for QueueSettings {
    fn default() -> Self {
        Self {
            max_global_concurrent: 0,
            max_per_host_concurrent: 2,
        }
    }
}

impl QueueSettings {
    pub fn global_limit(self) -> usize {
        if self.max_global_concurrent == 0 {
            24
        } else {
            self.max_global_concurrent.min(24)
        }
    }

    pub fn per_host_limit(self) -> usize {
        self.max_per_host_concurrent.clamp(1, 4)
    }
}

#[derive(Debug, Clone)]
pub struct QueueJob {
    pub id: u64,
    pub host: HostId,
    pub source_url: String,
    pub thumbnail_url: String,
    pub image_index_zero_based: usize,
    pub force_order: bool,
    pub destination_directory: std::path::PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QueueStatus {
    Pending,
    Running,
    Finished,
    Error,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueueMove {
    Top,
    Up,
    Down,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QueueItemSnapshot {
    pub id: u64,
    pub host: HostId,
    pub status: QueueStatus,
    pub queue_rank: Option<usize>,
    pub filename: Option<String>,
    pub destination_path: Option<String>,
    pub bytes_written: u64,
    pub expected_bytes: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QueueSnapshot {
    pub running: usize,
    pub pending: usize,
    pub remaining: usize,
    pub completed: usize,
    pub errors: usize,
    pub items: Vec<QueueItemSnapshot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QueueEventKind {
    Added,
    Started,
    Finished,
    Failed,
    Stopped,
    Reordered,
    Cleared,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QueueEvent {
    pub kind: QueueEventKind,
    pub item: Option<QueueItemSnapshot>,
    pub snapshot: QueueSnapshot,
}

#[derive(Debug, Error)]
pub enum QueueError {
    #[error("download queue is no longer running")]
    Closed,
    #[error("queue response channel closed")]
    ResponseClosed,
}

struct RunningJob {
    host: HostId,
    abort: AbortHandle,
}

enum Command {
    Add(QueueJob, oneshot::Sender<u64>),
    Stop(u64, oneshot::Sender<bool>),
    StopAll,
    Move(u64, QueueMove, oneshot::Sender<bool>),
    ClearFinished,
    Snapshot(oneshot::Sender<QueueSnapshot>),
    Shutdown,
}

#[derive(Clone)]
pub struct QueueManager {
    commands: mpsc::Sender<Command>,
    events: broadcast::Sender<QueueEvent>,
}

impl QueueManager {
    pub fn new(pipeline: HostPipeline, settings: QueueSettings) -> Self {
        let (commands, receiver) = mpsc::channel(128);
        let (events, _) = broadcast::channel(256);
        tokio::spawn(run_queue(pipeline, settings, receiver, events.clone()));
        Self { commands, events }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<QueueEvent> {
        self.events.subscribe()
    }

    pub async fn add(&self, mut job: QueueJob) -> Result<u64, QueueError> {
        let id = NEXT_JOB_ID.fetch_add(1, Ordering::Relaxed);
        job.id = id;
        let (reply, response) = oneshot::channel();
        self.commands
            .send(Command::Add(job, reply))
            .await
            .map_err(|_| QueueError::Closed)?;
        response.await.map_err(|_| QueueError::ResponseClosed)
    }

    pub async fn stop(&self, id: u64) -> Result<bool, QueueError> {
        let (reply, response) = oneshot::channel();
        self.commands
            .send(Command::Stop(id, reply))
            .await
            .map_err(|_| QueueError::Closed)?;
        response.await.map_err(|_| QueueError::ResponseClosed)
    }

    pub async fn stop_all(&self) -> Result<(), QueueError> {
        self.commands
            .send(Command::StopAll)
            .await
            .map_err(|_| QueueError::Closed)
    }

    pub async fn move_job(&self, id: u64, movement: QueueMove) -> Result<bool, QueueError> {
        let (reply, response) = oneshot::channel();
        self.commands
            .send(Command::Move(id, movement, reply))
            .await
            .map_err(|_| QueueError::Closed)?;
        response.await.map_err(|_| QueueError::ResponseClosed)
    }

    pub async fn clear_finished(&self) -> Result<(), QueueError> {
        self.commands
            .send(Command::ClearFinished)
            .await
            .map_err(|_| QueueError::Closed)
    }

    pub async fn snapshot(&self) -> Result<QueueSnapshot, QueueError> {
        let (reply, response) = oneshot::channel();
        self.commands
            .send(Command::Snapshot(reply))
            .await
            .map_err(|_| QueueError::Closed)?;
        response.await.map_err(|_| QueueError::ResponseClosed)
    }

    pub async fn shutdown(&self) -> Result<(), QueueError> {
        self.commands
            .send(Command::Shutdown)
            .await
            .map_err(|_| QueueError::Closed)
    }
}

async fn run_queue(
    pipeline: HostPipeline,
    settings: QueueSettings,
    mut commands: mpsc::Receiver<Command>,
    events: broadcast::Sender<QueueEvent>,
) {
    let mut pending = VecDeque::<QueueJob>::new();
    let mut running = HashMap::<u64, RunningJob>::new();
    let mut items = HashMap::<u64, QueueItemSnapshot>::new();
    let mut workers = JoinSet::<(u64, Result<CompletedImage, PipelineError>)>::new();
    let global_limit = settings.global_limit();
    let per_host_limit = settings.per_host_limit();

    loop {
        schedule_pending(
            &pipeline,
            &mut pending,
            &mut running,
            &mut items,
            &mut workers,
            global_limit,
            per_host_limit,
            &events,
        );

        tokio::select! {
            command = commands.recv() => {
                let Some(command) = command else { break; };
                match command {
                    Command::Add(job, reply) => {
                        let id = job.id;
                        items.insert(id, QueueItemSnapshot {
                            id,
                            host: job.host,
                            status: QueueStatus::Pending,
                            queue_rank: Some(pending.len() + 1),
                            filename: None,
                            destination_path: None,
                            bytes_written: 0,
                            expected_bytes: None,
                            error: None,
                        });
                        pending.push_back(job);
                        let _ = reply.send(id);
                        publish(&events, QueueEventKind::Added, items.get(&id).cloned(), &pending, &running, &items);
                    }
                    Command::Stop(id, reply) => {
                        let mut stopped = false;
                        if let Some(position) = pending.iter().position(|job| job.id == id) {
                            pending.remove(position);
                            stopped = true;
                        }
                        if let Some(active) = running.remove(&id) {
                            active.abort.abort();
                            stopped = true;
                        }
                        if stopped {
                            if let Some(item) = items.get_mut(&id) {
                                item.status = QueueStatus::Stopped;
                                item.queue_rank = None;
                            }
                            publish(&events, QueueEventKind::Stopped, items.get(&id).cloned(), &pending, &running, &items);
                        }
                        let _ = reply.send(stopped);
                    }
                    Command::StopAll => {
                        for job in pending.drain(..) {
                            if let Some(item) = items.get_mut(&job.id) {
                                item.status = QueueStatus::Stopped;
                                item.queue_rank = None;
                            }
                        }
                        for (id, active) in running.drain() {
                            active.abort.abort();
                            if let Some(item) = items.get_mut(&id) {
                                item.status = QueueStatus::Stopped;
                                item.queue_rank = None;
                            }
                        }
                        publish(&events, QueueEventKind::Stopped, None, &pending, &running, &items);
                    }
                    Command::Move(id, movement, reply) => {
                        let moved = move_pending(&mut pending, id, movement);
                        if moved {
                            update_pending_ranks(&pending, &mut items);
                            publish(&events, QueueEventKind::Reordered, items.get(&id).cloned(), &pending, &running, &items);
                        }
                        let _ = reply.send(moved);
                    }
                    Command::ClearFinished => {
                        items.retain(|_, item| item.status != QueueStatus::Finished);
                        publish(&events, QueueEventKind::Cleared, None, &pending, &running, &items);
                    }
                    Command::Snapshot(reply) => {
                        let _ = reply.send(make_snapshot(&pending, &running, &items));
                    }
                    Command::Shutdown => {
                        pending.clear();
                        for active in running.values() {
                            active.abort.abort();
                        }
                        break;
                    }
                }
            }
            completed = workers.join_next(), if !workers.is_empty() => {
                if let Some(result) = completed {
                    match result {
                        Ok((id, Ok(download))) => {
                            running.remove(&id);
                            if let Some(item) = items.get_mut(&id) {
                                item.status = if download.complete { QueueStatus::Finished } else { QueueStatus::Error };
                                item.filename = Some(download.filename);
                                item.destination_path = Some(download.path.to_string_lossy().into_owned());
                                item.bytes_written = download.bytes_written;
                                item.expected_bytes = download.expected_bytes;
                                if !download.complete {
                                    item.error = Some("downloaded byte count did not match the expected size".into());
                                }
                            }
                            let kind = if items.get(&id).map(|item| item.status) == Some(QueueStatus::Finished) {
                                QueueEventKind::Finished
                            } else {
                                QueueEventKind::Failed
                            };
                            publish(&events, kind, items.get(&id).cloned(), &pending, &running, &items);
                        }
                        Ok((id, Err(error))) => {
                            running.remove(&id);
                            if let Some(item) = items.get_mut(&id) {
                                item.status = QueueStatus::Error;
                                item.error = Some(error.to_string());
                            }
                            publish(&events, QueueEventKind::Failed, items.get(&id).cloned(), &pending, &running, &items);
                        }
                        Err(error) => {
                            let task_id = error.id();
                            if let Some(id) = running.iter().find_map(|(id, active)| {
                                (active.abort.id() == task_id).then_some(*id)
                            }) {
                                running.remove(&id);
                                if let Some(item) = items.get_mut(&id) {
                                    item.status = QueueStatus::Error;
                                    item.error = Some("download worker terminated unexpectedly".into());
                                }
                                publish(&events, QueueEventKind::Failed, items.get(&id).cloned(), &pending, &running, &items);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn schedule_pending(
    pipeline: &HostPipeline,
    pending: &mut VecDeque<QueueJob>,
    running: &mut HashMap<u64, RunningJob>,
    items: &mut HashMap<u64, QueueItemSnapshot>,
    workers: &mut JoinSet<(u64, Result<CompletedImage, PipelineError>)>,
    global_limit: usize,
    per_host_limit: usize,
    events: &broadcast::Sender<QueueEvent>,
) {
    while running.len() < global_limit {
        let mut host_counts = HashMap::<HostId, usize>::new();
        for active in running.values() {
            *host_counts.entry(active.host).or_default() += 1;
        }
        let Some(position) = pending
            .iter()
            .position(|job| host_counts.get(&job.host).copied().unwrap_or(0) < per_host_limit)
        else {
            break;
        };
        let Some(job) = pending.remove(position) else {
            break;
        };

        if let Some(item) = items.get_mut(&job.id) {
            item.status = QueueStatus::Running;
            item.queue_rank = None;
        }
        let id = job.id;
        let host = job.host;
        let worker_pipeline = pipeline.clone();
        let worker_job = job.clone();
        let abort = workers.spawn(async move {
            let result = worker_pipeline
                .download_image(
                    worker_job.host,
                    &worker_job.source_url,
                    &worker_job.thumbnail_url,
                    worker_job.image_index_zero_based,
                    worker_job.force_order,
                    &worker_job.destination_directory,
                )
                .await;
            (worker_job.id, result)
        });
        running.insert(id, RunningJob { host, abort });
        update_pending_ranks(pending, items);
        publish(events, QueueEventKind::Started, items.get(&id).cloned(), pending, running, items);
    }
}

fn move_pending(pending: &mut VecDeque<QueueJob>, id: u64, movement: QueueMove) -> bool {
    let Some(index) = pending.iter().position(|job| job.id == id) else {
        return false;
    };
    let job = pending.remove(index).expect("position was found");
    let destination = match movement {
        QueueMove::Top => 0,
        QueueMove::Up => index.saturating_sub(1),
        QueueMove::Down => (index + 1).min(pending.len()),
        QueueMove::Bottom => pending.len(),
    };
    pending.insert(destination, job);
    true
}

fn update_pending_ranks(
    pending: &VecDeque<QueueJob>,
    items: &mut HashMap<u64, QueueItemSnapshot>,
) {
    for (index, job) in pending.iter().enumerate() {
        if let Some(item) = items.get_mut(&job.id) {
            item.queue_rank = Some(index + 1);
        }
    }
}

fn make_snapshot(
    pending: &VecDeque<QueueJob>,
    running: &HashMap<u64, RunningJob>,
    items: &HashMap<u64, QueueItemSnapshot>,
) -> QueueSnapshot {
    let mut ordered = Vec::new();
    for job in pending {
        if let Some(item) = items.get(&job.id) {
            let mut item = item.clone();
            item.queue_rank = Some(ordered.len() + 1);
            ordered.push(item);
        }
    }
    let mut running_ids: Vec<_> = running.keys().copied().collect();
    running_ids.sort_unstable();
    for id in running_ids {
        if let Some(item) = items.get(&id) {
            ordered.push(item.clone());
        }
    }
    let mut completed: Vec<_> = items
        .values()
        .filter(|item| {
            !matches!(
                item.status,
                QueueStatus::Pending | QueueStatus::Running
            )
        })
        .cloned()
        .collect();
    completed.sort_by_key(|item| item.id);
    ordered.extend(completed);

    let completed_count = items
        .values()
        .filter(|item| item.status == QueueStatus::Finished)
        .count();
    let error_count = items
        .values()
        .filter(|item| item.status == QueueStatus::Error)
        .count();
    QueueSnapshot {
        running: running.len(),
        pending: pending.len(),
        remaining: pending.len() + running.len(),
        completed: completed_count,
        errors: error_count,
        items: ordered,
    }
}

fn publish(
    events: &broadcast::Sender<QueueEvent>,
    kind: QueueEventKind,
    item: Option<QueueItemSnapshot>,
    pending: &VecDeque<QueueJob>,
    running: &HashMap<u64, RunningJob>,
    items: &HashMap<u64, QueueItemSnapshot>,
) {
    let _ = events.send(QueueEvent {
        kind,
        item,
        snapshot: make_snapshot(pending, running, items),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::NetworkConfig;

    #[test]
    fn zero_global_limit_uses_reference_pool_fallback() {
        assert_eq!(QueueSettings::default().global_limit(), 24);
        assert_eq!(QueueSettings { max_global_concurrent: 7, max_per_host_concurrent: 2 }.global_limit(), 7);
    }

    #[test]
    fn per_host_limit_is_clamped_to_reference_range() {
        assert_eq!(QueueSettings { max_global_concurrent: 2, max_per_host_concurrent: 0 }.per_host_limit(), 1);
        assert_eq!(QueueSettings { max_global_concurrent: 2, max_per_host_concurrent: 8 }.per_host_limit(), 4);
    }


    #[test]
    fn moves_pending_jobs_to_top_and_bottom() {
        let mut pending = VecDeque::from(vec![
            QueueJob { id: 1, host: HostId::DPicMe, source_url: String::new(), thumbnail_url: String::new(), image_index_zero_based: 0, force_order: false, destination_directory: std::path::PathBuf::new() },
            QueueJob { id: 2, host: HostId::DPicMe, source_url: String::new(), thumbnail_url: String::new(), image_index_zero_based: 0, force_order: false, destination_directory: std::path::PathBuf::new() },
            QueueJob { id: 3, host: HostId::DPicMe, source_url: String::new(), thumbnail_url: String::new(), image_index_zero_based: 0, force_order: false, destination_directory: std::path::PathBuf::new() },
        ]);
        assert!(move_pending(&mut pending, 3, QueueMove::Top));
        assert_eq!(pending.iter().map(|job| job.id).collect::<Vec<_>>(), vec![3, 1, 2]);
        assert!(move_pending(&mut pending, 3, QueueMove::Bottom));
        assert_eq!(pending.iter().map(|job| job.id).collect::<Vec<_>>(), vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn queue_runs_a_job_through_the_download_pipeline() {
        use std::time::Duration;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("HEAD"))
            .and(path("/queued.jpg"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "image/jpeg")
                    .insert_header("content-length", "4"),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/queued.jpg"))
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
        let queue = QueueManager::new(pipeline, QueueSettings::default());
        let id = queue
            .add(QueueJob {
                id: 0,
                host: HostId::DPicMe,
                source_url: format!("{}/queued.jpg", server.uri()),
                thumbnail_url: String::new(),
                image_index_zero_based: 0,
                force_order: false,
                destination_directory: temp.path().join("downloads"),
            })
            .await
            .unwrap();

        let snapshot = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let snapshot = queue.snapshot().await.unwrap();
                if snapshot.items.iter().any(|item| {
                    item.id == id
                        && matches!(item.status, QueueStatus::Finished | QueueStatus::Error)
                }) {
                    break snapshot;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();

        let item = snapshot.items.iter().find(|item| item.id == id).unwrap();
        assert_eq!(item.status, QueueStatus::Finished);
        assert_eq!(item.filename.as_deref(), Some("queued.JPG"));
        queue.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn new_queue_starts_empty() {
        let temp = tempfile::tempdir().unwrap();
        let pipeline = HostPipeline::new(
            &NetworkConfig::default(),
            None,
            None,
            temp.path(),
            false,
        )
        .unwrap();
        let queue = QueueManager::new(pipeline, QueueSettings::default());
        let snapshot = queue.snapshot().await.unwrap();
        assert_eq!(snapshot.running, 0);
        assert_eq!(snapshot.pending, 0);
        assert_eq!(snapshot.remaining, 0);
        queue.shutdown().await.unwrap();
    }
}
