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
  Failed, // TODO: error message
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
}
