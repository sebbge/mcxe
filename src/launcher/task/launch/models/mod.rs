use super::super::{CancelFlag, InnerTask, TaskHandle};
use serde::Serialize;
use std::{sync::Arc, time::Instant};
use time::OffsetDateTime;
use tokio::sync::RwLock;
use tokio::sync::watch::Sender;

#[derive(typed_builder::TypedBuilder)]
pub struct LaunchConfig {
  #[builder(setter(into))]
  pub(crate) id: String,
  #[builder(setter(into))]
  pub(crate) version: String,
  #[builder(setter(into))]
  pub(super) username: String,
  #[builder(setter(into))]
  pub(super) uuid: String,
  #[builder(default)]
  pub(super) token: Option<String>,
  #[builder(setter(into))]
  pub(super) directory: String,
  #[builder(default)]
  pub(super) custom_java: Option<std::path::PathBuf>,
  #[builder(default)]
  pub(super) extra_jvm_args: Option<Vec<String>>,
  #[builder(default, setter(strip_option))]
  pub(super) extra_game_args: Option<Vec<String>>,
  #[builder(default)]
  #[allow(dead_code)] // TODO: use check_assets field
  pub(super) check_assets: Option<bool>,
}

//--- Public process state
#[derive(Serialize, Default, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ProcessStatus {
  #[default]
  Processing,
  Launching,
  Playing,
  Failed,
  Exited,
  Crashed,
}

#[derive(Default, Clone)]
pub struct Process {
  pub id: String,
  pub status: ProcessStatus,
  pub stdout: Vec<String>,
  pub stderr: Vec<String>,
  pub error: Option<String>,
  pub finish: Option<OffsetDateTime>,
  pub started_at: Option<Instant>,
}
impl Process {
  fn new(id: String) -> Self {
    Self { id, ..Self::default() }
  }

  fn start(&mut self, tx: &Sender<Self>) {
    self.status = ProcessStatus::Processing;
    self.started_at = Some(Instant::now());

    self.send_update(tx);
  }

  fn set_playing(&mut self, tx: &Sender<Self>) {
    self.status = ProcessStatus::Playing;

    self.send_update(tx);
  }

  pub fn add_line_stdout(&mut self, tx: &Sender<Self>, line: String) {
    self.stdout.push(line);

    self.send_update(tx);
  }

  pub fn add_line_stderr(&mut self, tx: &Sender<Self>, line: String) {
    self.stderr.push(line);

    self.send_update(tx);
  }

  fn fail(&mut self, tx: &Sender<Self>, error: String) {
    self.status = ProcessStatus::Failed;
    self.finish = Some(OffsetDateTime::now_utc());
    self.error = Some(error);

    self.send_update(tx);
  }

  fn exit(&mut self, tx: &Sender<Self>) {
    self.status = ProcessStatus::Exited;
    self.finish = Some(OffsetDateTime::now_utc());

    self.send_update(tx);
  }

  fn crash(&mut self, tx: &Sender<Self>) {
    self.status = ProcessStatus::Crashed;
    self.finish = Some(OffsetDateTime::now_utc());

    self.send_update(tx);
  }

  fn send_update(&self, tx: &Sender<Self>) {
    let _ = tx.send(self.clone());
  }
}

//--- Private process manager
#[derive(Clone)]
pub(crate) struct ProcessTask(Arc<InnerTask<Process>>);
impl ProcessTask {
  pub async fn new(id: impl Into<String>) -> (Self, TaskHandle<Process>) {
    let id = id.into();

    let process = Process::new(id.clone());
    let state = Arc::new(RwLock::new(process));
    let (tx, rx) = tokio::sync::watch::channel(state.read().await.clone());

    let cancel = CancelFlag::default();
    let handle = TaskHandle {
      id,
      rx,
      cancel: cancel.clone(),
    };

    let process = Arc::new(InnerTask { state, tx, cancel });
    process.state.write().await.send_update(&process.tx);
    (Self(process), handle)
  }

  pub async fn start(&self) {
    self.0.state.write().await.start(&self.0.tx);
  }

  pub async fn set_playing(&self) {
    self.0.state.write().await.set_playing(&self.0.tx);
  }

  pub async fn add_line_stdout(&self, line: String) {
    self.0.state.write().await.add_line_stdout(&self.0.tx, line);
  }

  pub async fn add_line_stderr(&self, line: String) {
    self.0.state.write().await.add_line_stderr(&self.0.tx, line);
  }

  pub async fn fail(&self, error: String) {
    self.0.state.write().await.fail(&self.0.tx, error);
  }

  pub async fn crash(&self) {
    self.0.state.write().await.crash(&self.0.tx);
  }

  pub async fn exit(&self) {
    self.0.state.write().await.exit(&self.0.tx);
  }
  //

  pub fn is_killed(&self) -> bool {
    self.0.cancel.is_cancelled()
  }
}
