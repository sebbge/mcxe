use super::super::{CancelFlag, InnerTask, TaskHandle};
use serde::Serialize;
use std::{sync::Arc, time::Instant};
use time::OffsetDateTime;
use tokio::sync::RwLock;
use tokio::sync::watch::Sender;

//--- Public operation state
#[derive(Serialize, Default, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum OperationStatus {
  #[default]
  Processing,
  InProgress,
  Failed,
  Cancelled,
  Completed,
}

#[derive(Serialize, Default, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum OperationKind {
  #[default]
  Unknown,
  Repairing,
  // ??
}

#[derive(Default, Clone)]
pub struct Operation {
  pub id: String,
  pub kind: OperationKind,
  pub status: OperationStatus,
  pub message: String,
  pub error: Option<String>,
  pub out: Option<String>,
  pub finish: Option<OffsetDateTime>,
  pub started_at: Option<Instant>,
}
impl Operation {
  fn new(id: String, kind: OperationKind) -> Self {
    Self {
      id,
      kind,
      ..Self::default()
    }
  }

  fn start(&mut self, tx: &Sender<Self>) {
    self.status = OperationStatus::InProgress;
    self.started_at = Some(Instant::now());

    self.send_update(tx);
  }

  fn set_message(&mut self, tx: &Sender<Self>, message: impl Into<String>) {
    self.message = message.into();

    self.send_update(tx);
  }

  fn fail(&mut self, tx: &Sender<Self>, error: String) {
    self.status = OperationStatus::Failed;
    self.finish = Some(OffsetDateTime::now_utc());
    self.error = Some(error);

    self.send_update(tx);
  }

  fn cancel(&mut self, tx: &Sender<Self>) {
    self.status = OperationStatus::Cancelled;
    self.finish = Some(OffsetDateTime::now_utc());

    self.send_update(tx);
  }

  fn complete(&mut self, tx: &Sender<Self>, out: Option<String>) {
    self.status = OperationStatus::Completed;
    self.finish = Some(OffsetDateTime::now_utc());
    self.out = out;

    self.send_update(tx);
  }

  fn send_update(&self, tx: &Sender<Self>) {
    let _ = tx.send(self.clone());
  }
}

//- Private operation manager
#[derive(Clone)]
pub(crate) struct OperationTask(Arc<InnerTask<Operation>>);
impl OperationTask {
  pub async fn new(id: impl Into<String>, kind: OperationKind) -> (Self, TaskHandle<Operation>) {
    let id = id.into();

    let operation = Operation::new(id.clone(), kind);
    let state = Arc::new(RwLock::new(operation));
    let (tx, rx) = tokio::sync::watch::channel(state.read().await.clone());

    let cancel = CancelFlag::default();
    let handle = TaskHandle {
      id,
      rx,
      cancel: cancel.clone(),
    };

    let operation = Arc::new(InnerTask { state, tx, cancel });
    operation.state.write().await.send_update(&operation.tx);
    (Self(operation), handle)
  }

  pub async fn start(&self) {
    self.0.state.write().await.start(&self.0.tx);
  }

  pub async fn set_message(&self, message: impl Into<String>) {
    self.0.state.write().await.set_message(&self.0.tx, message);
  }

  pub async fn fail(&self, error: String) {
    self.0.state.write().await.fail(&self.0.tx, error);
  }

  pub async fn cancel(&self) {
    self.0.state.write().await.cancel(&self.0.tx);
  }

  pub async fn complete(&self, out: Option<String>) {
    self.0.state.write().await.complete(&self.0.tx, out);
  }
  //

  pub fn is_cancelled(&self) -> bool {
    self.0.cancel.is_cancelled()
  }
}
