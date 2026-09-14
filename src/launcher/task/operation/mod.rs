mod checksum;
pub use checksum::*;

use serde::Serialize;
use taskee::{Task, TaskStatus};

// operation task
#[cfg_attr(feature = "ts-types", derive(ts_rs::TS), ts(export))]
#[derive(TaskStatus, Serialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ProgressStatus {
  #[default]
  Pending,
  Running,
  #[terminal]
  Failed,
  #[terminal]
  Aborted,
  #[terminal]
  Success,
}

#[derive(Task, Default, Clone)]
#[taskee(progress, status)]
pub struct Progress {
  pub status: ProgressStatus,
  pub current: u64,
  pub total: u64,
  pub progress: u8, // (0 - 100)
  pub message: String,
  pub error_message: String,
  pub out: Option<String>,
}

impl ProgressTask {
  pub fn set_message(&self, message: impl Into<String>) {
    self.set(|payload| payload.message = message.into());
  }

  pub fn set_error_message(&self, message: impl Into<String>) {
    self.set(|payload| payload.error_message = message.into());
  }

  pub fn set_out(&self, out: Option<String>) {
    self.set(|payload| payload.out = out);
  }
}
