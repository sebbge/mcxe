mod download;
pub use download::*;

mod launch;
pub use launch::*;

mod operation;
pub use operation::*;

use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{sync::Arc, time::SystemTime};
use tokio::sync::RwLock;
use tokio::sync::watch::{Receiver, Sender, channel};

#[derive(Serialize, Default, Debug, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TaskStatus {
  #[default]
  Pending,
  InProgress, // downloading/launching
  Running,    // installing/playing
  Failed,     // failed/failed to launch
  Aborted,    // cancelled/crashed
  Success,    // completed/exited manually
}

#[derive(Serialize, Default, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TaskKind {
  #[default]
  Download,
  Process,
  Repair,
}

#[derive(Default, Clone)]
pub struct Task<T> {
  pub id: String,
  pub kind: TaskKind,
  pub status: TaskStatus,
  pub payload: T,
  pub message: String,
  pub error_message: Option<String>,
  pub finished_at: Option<SystemTime>,
  pub started_at: Option<SystemTime>,
}
impl<T: Default + Clone> Task<T> {
  fn new(id: String, kind: TaskKind) -> Self {
    Self {
      id,
      kind,
      ..Self::default()
    }
  }

  fn start(&mut self, tx: &Sender<Self>) {
    self.started_at = Some(SystemTime::now());
    self.status = TaskStatus::InProgress;
    self.send_update(tx);
  }

  fn set_status(&mut self, tx: &Sender<Self>, status: TaskStatus) {
    self.status = status;
    self.send_update(tx);
  }

  fn set_message(&mut self, tx: &Sender<Self>, message: impl Into<String>) {
    self.message = message.into();
    self.send_update(tx);
  }

  fn fail(&mut self, tx: &Sender<Self>, error: String) {
    self.finished_at = Some(SystemTime::now());
    self.status = TaskStatus::Failed;
    self.error_message = Some(error);
    self.send_update(tx);
  }

  fn abort(&mut self, tx: &Sender<Self>) {
    self.finished_at = Some(SystemTime::now());
    self.status = TaskStatus::Aborted;
    self.send_update(tx);
  }

  fn success(&mut self, tx: &Sender<Self>) {
    self.finished_at = Some(SystemTime::now());
    self.status = TaskStatus::Success;
    self.send_update(tx);
  }

  fn send_update(&self, tx: &Sender<Self>) {
    tx.send(self.clone()).ok();
  }
}

//- Public task handle
#[derive(Clone)]
pub struct TaskHandle<T> {
  pub id: String,
  pub rx: Receiver<Task<T>>,
  abort: AbortFlag,
}
impl<T> TaskHandle<T> {
  pub fn abort(&self) {
    self.abort.abort();
  }
}

//- Private task manager
#[derive(Clone)]
struct InnerTask<T> {
  state: Arc<RwLock<Task<T>>>,
  tx: Sender<Task<T>>,
  abort: AbortFlag,
}

#[derive(Clone)]
pub(crate) struct TaskManager<T>(Arc<InnerTask<T>>);
impl<T: Default + Clone> TaskManager<T> {
  pub async fn new(id: impl Into<String>, kind: TaskKind) -> (Self, TaskHandle<T>) {
    let id = id.into();

    let task = Task::new(id.clone(), kind);
    let state = Arc::new(RwLock::new(task));
    let (tx, rx) = channel(state.read().await.clone());

    let abort = AbortFlag::default();

    let handle = TaskHandle {
      id,
      rx,
      abort: abort.clone(),
    };
    let task = Arc::new(InnerTask { state, tx, abort });

    task.state.write().await.send_update(&task.tx);

    (Self(task), handle)
  }

  pub async fn start(&self) {
    self.0.state.write().await.start(&self.0.tx);
  }

  pub async fn set_message(&self, message: impl Into<String>) {
    self.0.state.write().await.set_message(&self.0.tx, message);
  }

  pub async fn fail(&self, error_message: String) {
    self.0.state.write().await.fail(&self.0.tx, error_message);
  }

  pub async fn abort(&self) {
    self.0.abort.abort();
    self.0.state.write().await.abort(&self.0.tx);
  }

  pub async fn success(&self) {
    self.0.state.write().await.success(&self.0.tx);
  }

  pub fn was_aborted(&self) -> bool {
    self.0.abort.was_aborted()
  }
}

#[derive(Default, Clone)]
struct AbortFlag(Arc<AtomicBool>);
impl AbortFlag {
  fn abort(&self) {
    self.0.store(true, Ordering::Relaxed);
  }

  fn was_aborted(&self) -> bool {
    self.0.load(Ordering::Relaxed)
  }
}
