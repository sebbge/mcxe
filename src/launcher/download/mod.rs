use serde::Serialize;
use std::time::Instant;
use time::OffsetDateTime;

#[derive(Serialize, Default, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum DownloadStatus {
  #[default]
  Processing,
  InProgress,
  Installing,
  Completed,
  Failed,
  Cancelled,
}

#[derive(Clone)]
pub struct Download {
  id: String,
  status: DownloadStatus,
  message: String,
  total_bytes: usize,
  downloaded_bytes: usize,
  progress: f32, // (0.0 - 100.0)
  start: Option<OffsetDateTime>,
  finish: Option<OffsetDateTime>,
  started_at: Option<Instant>,
}
impl Download {
  pub fn new(id: impl Into<String>) -> Self {
    Self {
      id: id.into(),
      status: DownloadStatus::default(),
      message: String::new(),
      total_bytes: 0,
      downloaded_bytes: 0,
      progress: 0.0,
      start: None,
      finish: None,
      started_at: None,
    }
  }

  pub fn start(&mut self) {
    self.status = DownloadStatus::InProgress;
    self.start = Some(OffsetDateTime::now_utc());
    self.started_at = Some(Instant::now());
  }

  pub fn set_total(&mut self, total: usize) {
    self.total_bytes = total
  }

  pub fn set_message(&mut self, message: impl Into<String>) {
    self.message = message.into();
  }

  pub fn update_progress(&mut self, bytes: usize) {
    self.downloaded_bytes = self.downloaded_bytes.saturating_add(bytes).min(self.total_bytes);

    self.progress = if self.total_bytes == 0 {
      0.0
    } else {
      (self.downloaded_bytes as f32 / self.total_bytes as f32) * 100.0
    };

    self.progress = self.progress.min(100.0);
  }

  pub fn set_installing(&mut self) {
    self.status = DownloadStatus::Installing;
  }

  pub fn fail(&mut self, message: impl Into<String>) {
    self.status = DownloadStatus::Failed;
    self.finish = Some(OffsetDateTime::now_utc());
    self.message = message.into();
  }

  pub fn finish(&mut self, message: impl Into<String>) {
    self.status = DownloadStatus::Completed;
    self.finish = Some(OffsetDateTime::now_utc());
    self.message = message.into();
  }

  pub fn send_update(&self, tx: &tokio::sync::watch::Sender<Download>) {
    let _ = tx.send(self.clone());
  }
}
