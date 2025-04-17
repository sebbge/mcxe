use serde::Serialize;
use std::time::Instant;
use time::OffsetDateTime;
use tokio::sync::watch::Sender;

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

  pub fn start(&mut self, tx: &Sender<Download>) {
    self.status = DownloadStatus::InProgress;
    self.start = Some(OffsetDateTime::now_utc());
    self.started_at = Some(Instant::now());

    self.send_update(tx);
  }

  pub fn set_total(&mut self, total: u64, tx: &Sender<Download>) {
    self.total_bytes = total;

    self.send_update(tx);
  }

  pub fn set_message(&mut self, message: impl Into<String>, tx: &Sender<Download>) {
    self.message = message.into();

    self.send_update(tx);
  }

  pub fn add_bytes(&mut self, bytes: u64, tx: &Sender<Download>) {
    self.downloaded_bytes = self.downloaded_bytes.saturating_add(bytes).min(self.total_bytes);

    self.progress = if self.total_bytes == 0 {
      0.0
    } else {
      (self.downloaded_bytes as f32 / self.total_bytes as f32) * 100.0
    };

    self.progress = self.progress.min(100.0);

    self.send_update(tx);
  }

  pub fn set_installing(&mut self, tx: &Sender<Download>) {
    self.status = DownloadStatus::Installing;

    self.send_update(tx);
  }

  pub fn fail(&mut self, message: impl Into<String>, tx: &Sender<Download>) {
    self.status = DownloadStatus::Failed;
    self.finish = Some(OffsetDateTime::now_utc());
    self.message = message.into();

    self.send_update(tx);
  }

  pub fn finish(&mut self, message: impl Into<String>, tx: &Sender<Download>) {
    self.status = DownloadStatus::Completed;
    self.finish = Some(OffsetDateTime::now_utc());
    self.message = message.into();

    self.send_update(tx);
  }

  fn send_update(&self, tx: &Sender<Download>) {
    let _ = tx.send(self.clone());
  }
}
