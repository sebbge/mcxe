use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{sync::Arc, time::Instant};
use time::OffsetDateTime;
use tokio::sync::RwLock;
use tokio::sync::watch::{Receiver, Sender};

//- Download tasks
pub(super) enum TaskResult {
  Cancelled,
  Impossible,
  Ignored,
  Success,
}
pub(super) type Task<T> = tokio::task::JoinHandle<(T, crate::Result<TaskResult>)>;

//- Public download state
#[derive(Serialize, Default, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum DownloadStatus {
  #[default]
  Processing,
  Downloading,
  Installing,
  Completed,
  Failed,
  Cancelled,
}

#[derive(Default, Clone)]
pub struct DownloadState {
  pub id: String,
  pub status: DownloadStatus,
  pub message: String,
  pub total_bytes: u64,
  pub downloaded_bytes: u64,
  pub progress: f32, // (0.0 - 100.0)
  pub start: Option<OffsetDateTime>,
  pub finish: Option<OffsetDateTime>,
  pub started_at: Option<Instant>,
}
impl DownloadState {
  fn new(id: String) -> Self {
    Self { id, ..Self::default() }
  }

  fn start(&mut self, tx: &Sender<Self>) {
    self.status = DownloadStatus::Downloading;
    self.start = Some(OffsetDateTime::now_utc());
    self.started_at = Some(Instant::now());

    self.send_update(tx);
  }

  fn set_total(&mut self, tx: &Sender<Self>, total: u64) {
    self.total_bytes = total;

    self.send_update(tx);
  }

  fn set_message(&mut self, tx: &Sender<Self>, message: impl Into<String>) {
    self.message = message.into();

    self.send_update(tx);
  }

  fn add_bytes(&mut self, tx: &Sender<Self>, bytes: usize, message: impl Into<String>) {
    let prev = self.progress.floor();

    self.downloaded_bytes = self.downloaded_bytes.saturating_add(bytes as u64).min(self.total_bytes);
    self.message = message.into();

    self.progress = if self.total_bytes == 0 {
      0.0
    } else {
      (self.downloaded_bytes as f32 / self.total_bytes as f32) * 100.0
    };

    self.progress = self.progress.min(100.0);

    if prev < self.progress.floor() {
      self.send_update(tx);
    }
  }

  fn set_installing(&mut self, tx: &Sender<Self>, message: impl Into<String>) {
    self.status = DownloadStatus::Installing;
    self.message = message.into();

    self.send_update(tx);
  }

  fn fail(&mut self, tx: &Sender<Self>, message: impl Into<String>) {
    self.status = DownloadStatus::Failed;
    self.finish = Some(OffsetDateTime::now_utc());
    self.message = message.into();

    self.send_update(tx);
  }

  fn finish(&mut self, tx: &Sender<Self>, message: impl Into<String>) {
    self.status = DownloadStatus::Completed;
    self.progress = 100.0;
    self.finish = Some(OffsetDateTime::now_utc());
    self.message = message.into();

    self.send_update(tx);
  }

  fn cancel(&mut self, tx: &Sender<Self>, message: impl Into<String>) {
    self.status = DownloadStatus::Cancelled;
    self.finish = Some(OffsetDateTime::now_utc());
    self.message = message.into();

    self.send_update(tx);
  }

  fn send_update(&self, tx: &Sender<Self>) {
    let _ = tx.send(self.clone());
  }
}

//- Public download handle
#[derive(Clone)]
pub struct DownloadHandle {
  pub id: String,
  pub rx: Receiver<DownloadState>,
  cancel: CancelFlag,
}
impl DownloadHandle {
  pub fn cancel(&self) {
    self.cancel.cancel();
  }
}

//- Private download manager
struct InnerDownload {
  state: Arc<RwLock<DownloadState>>,
  tx: Sender<DownloadState>,
  cancel: CancelFlag,
}

#[derive(Clone)]
pub(crate) struct Download(Arc<InnerDownload>);
impl Download {
  pub async fn new(id: impl Into<String>) -> (Self, DownloadHandle) {
    let id = id.into();

    let download = DownloadState::new(id.clone());
    let state = Arc::new(RwLock::new(download));
    let (tx, rx) = tokio::sync::watch::channel(state.read().await.clone());

    let cancel = CancelFlag::default();
    let handle = DownloadHandle {
      id,
      rx,
      cancel: cancel.clone(),
    };

    let download = Arc::new(InnerDownload { state, tx, cancel });
    download.state.write().await.send_update(&download.tx);
    (Self(download), handle)
  }

  pub async fn start(&self) {
    self.0.state.write().await.start(&self.0.tx);
  }

  pub async fn set_total(&self, total: u64) {
    self.0.state.write().await.set_total(&self.0.tx, total);
  }

  pub async fn set_message(&self, message: impl Into<String>) {
    self.0.state.write().await.set_message(&self.0.tx, message);
  }

  pub async fn add_bytes(&self, bytes: usize, message: impl Into<String>) {
    self.0.state.write().await.add_bytes(&self.0.tx, bytes, message);
  }

  pub async fn set_installing(&self, message: impl Into<String>) {
    self.0.state.write().await.set_installing(&self.0.tx, message);
  }

  pub async fn fail(&self, message: impl Into<String>) {
    self.0.state.write().await.fail(&self.0.tx, message);
  }

  pub async fn finish(&self, message: impl Into<String>) {
    self.0.state.write().await.finish(&self.0.tx, message);
  }

  //

  pub async fn cancel(&self, message: impl Into<String>) {
    self.0.cancel.cancel();
    self.0.state.write().await.cancel(&self.0.tx, message);
  }

  pub fn is_cancelled(&self) -> bool {
    self.0.cancel.is_cancelled()
  }
}

//- Download cancel flag
#[derive(Default, Clone)]
struct CancelFlag(Arc<AtomicBool>);
impl CancelFlag {
  fn cancel(&self) {
    self.0.store(true, Ordering::Relaxed);
  }

  fn is_cancelled(&self) -> bool {
    self.0.load(Ordering::Relaxed)
  }
}
