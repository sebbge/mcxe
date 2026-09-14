mod game;
mod java;
pub(crate) use game::*;
pub(crate) use java::*;

use serde::Serialize;
use taskee::{Task, TaskStatus};

// download jobs
pub(super) enum DownloadResult {
  Cancelled,
  Impossible,
  Ignored,
  Success,
}
pub(super) type DownloadJob<T> = tokio::task::JoinHandle<(T, crate::Result<DownloadResult>)>;

// download task
#[cfg_attr(feature = "ts-types", derive(ts_rs::TS), ts(export))]
#[derive(TaskStatus, Serialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
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
#[taskee(progress, status)]
pub struct Download {
  pub status: DownloadStatus,
  pub current: u64, // bytes downloaded
  pub total: u64,   // total bytes
  pub progress: u8, // 0-100
  pub message: String,
  pub error_message: String,
}

impl DownloadTask {
  pub fn set_message(&self, message: impl Into<String>) {
    self.set(|payload| payload.message = message.into());
  }

  pub fn set_error_message(&self, message: impl Into<String>) {
    self.set(|payload| payload.error_message = message.into());
  }

  pub fn add_progress_msg(&self, amount: u64, message: impl Into<String>) {
    self.advance_with(amount, |payload| payload.message = message.into());
  }
}
