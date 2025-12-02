mod game;
mod java;
pub(crate) use game::*;
pub(crate) use java::*;

use serde::Serialize;
use tasked::{Task, TaskStatus};

// download jobs
pub(super) enum DownloadResult {
  Cancelled,
  Impossible,
  Ignored,
  Success,
}
pub(super) type DownloadJob<T> = tokio::task::JoinHandle<(T, crate::Result<DownloadResult>)>;

// download task
#[derive(TaskStatus, Serialize, Default, Clone, Debug, PartialEq)]
pub enum DownloadStatus {
  #[default]
  Pending,
  Downloading,
  Installing,
  #[terminal]
  Failed,
  #[terminal]
  Aborted,
  #[terminal]
  Success,
}

#[derive(Task, Default, Clone)]
#[tasked(progress, status)]
pub struct Download {
  pub id: String,
  pub status: DownloadStatus,
  pub current: u64, // bytes downloaded
  pub total: u64,   // total bytes
  pub progress: u8, // 0-100
  pub message: String,
  pub error_message: String,
}

// private download manager
impl DownloadTask {
  pub async fn set_message(&self, message: impl Into<String>) {
    self
      .update(|payload| {
        payload.message = message.into();
        true
      })
      .await;
  }

  pub async fn set_error_message(&self, message: impl Into<String>) {
    self
      .update(|payload| {
        payload.error_message = message.into();
        true
      })
      .await;
  }

  pub async fn add_progress_msg(&self, amount: u64, message: impl Into<String>) {
    use tasked::Progress;
    self
      .update(|payload| {
        let prev_progress = payload.progress();

        let new_current = payload.current().saturating_add(amount).min(payload.total());
        payload.set_current(new_current);

        let new_progress = payload.calc_progress();
        payload.set_progress(new_progress);
        payload.message = message.into();

        prev_progress < new_progress
      })
      .await;
  }
}
