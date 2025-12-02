mod checksum;
pub use checksum::*;

use serde::Serialize;
use tasked::{Task, TaskStatus};

// operation task
#[derive(TaskStatus, Serialize, Default, Clone, Debug)]
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
#[tasked(progress, status)]
pub struct Progress {
  pub id: String,
  pub status: ProgressStatus,
  pub current: u64,
  pub total: u64,
  pub progress: u8, // (0 - 100)
  pub message: String,
  pub error_message: String,
  pub out: Option<String>,
}

// internal progress manager
impl ProgressTask {
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

  pub async fn set_out(&self, out: Option<String>) {
    self
      .update(|payload| {
        payload.out = out;
        true
      })
      .await;
  }
}
