use super::super::{Task, TaskManager, TaskStatus};
use tokio::sync::watch::Sender;

//--- Download jobs
pub(super) enum DownloadResult {
  Cancelled,
  Impossible,
  Ignored,
  Success,
}
pub(super) type DownloadJob<T> = tokio::task::JoinHandle<(T, crate::Result<DownloadResult>)>;

//--- Public download states
#[derive(Default, Clone)]
pub struct Download {
  pub id: String,
  pub total_bytes: u64,
  pub downloaded_bytes: u64,
  pub progress: u8, // (0 - 100)
}
impl Task<Download> {
  fn set_total(&mut self, tx: &Sender<Self>, total: u64) {
    self.payload.total_bytes = total;

    self.send_update(tx);
  }

  fn add_bytes(&mut self, tx: &Sender<Self>, bytes: usize, message: impl Into<String>) {
    let payload = &mut self.payload;
    let prev = payload.progress;

    payload.downloaded_bytes = payload
      .downloaded_bytes
      .saturating_add(bytes as u64)
      .min(payload.total_bytes);
    self.message = message.into();

    payload.progress = if payload.total_bytes == 0 {
      0
    } else {
      ((payload.downloaded_bytes as f32 / payload.total_bytes as f32) * 100.0).min(100.0) as u8
    };

    payload.progress = payload.progress.min(100);

    if prev < payload.progress {
      self.send_update(tx);
    }
  }
}

//--- Private download manager
impl TaskManager<Download> {
  pub async fn set_total(&self, total: u64) {
    self.0.state.write().await.set_total(&self.0.tx, total);
  }

  pub async fn add_bytes(&self, bytes: usize, message: impl Into<String>) {
    self.0.state.write().await.add_bytes(&self.0.tx, bytes, message);
  }

  pub async fn set_installing(&self) {
    self.0.state.write().await.set_status(&self.0.tx, TaskStatus::Running);
  }
}
