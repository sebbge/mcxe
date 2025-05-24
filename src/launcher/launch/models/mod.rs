use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{sync::Arc, time::Instant};
use time::OffsetDateTime;
use tokio::sync::RwLock;
use tokio::sync::watch::{Receiver, Sender};

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
  #[builder(default, setter(strip_option))]
  pub(super) token: Option<String>,
  #[builder(setter(into))]
  pub(super) directory: String,
  #[builder(default)]
  pub(super) custom_java: Option<PathBuf>,
  #[builder(default)]
  pub(super) extra_jvm_args: Option<Vec<String>>,
  #[builder(default, setter(strip_option))]
  pub(super) extra_game_args: Option<Vec<String>>,
  #[builder(default)]
  #[allow(dead_code)] // TODO: use check_assets field
  pub(super) check_assets: Option<bool>,
}

//- Public launch state
#[derive(Serialize, Default, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum LaunchStatus {
  #[default]
  Processing,
  Launching,
  Playing,
  Failed,
  Exited,
  Crashed,
}

#[derive(Default, Clone)]
pub struct LaunchState {
  pub id: String,
  pub status: LaunchStatus,
  pub stderr: Option<String>,
  pub finish: Option<OffsetDateTime>,
  pub started_at: Option<Instant>,
}
impl LaunchState {
  fn new(id: String) -> Self {
    Self { id, ..Self::default() }
  }

  fn start(&mut self, tx: &Sender<Self>) {
    self.status = LaunchStatus::Launching;
    self.started_at = Some(Instant::now());

    self.send_update(tx);
  }

  fn set_playing(&mut self, tx: &Sender<Self>) {
    self.status = LaunchStatus::Playing;

    self.send_update(tx);
  }

  fn fail(&mut self, tx: &Sender<Self>, error: String) {
    self.status = LaunchStatus::Failed;
    self.finish = Some(OffsetDateTime::now_utc());
    self.stderr = Some(error);

    self.send_update(tx);
  }

  fn exit(&mut self, tx: &Sender<Self>) {
    self.status = LaunchStatus::Exited;
    self.finish = Some(OffsetDateTime::now_utc());

    self.send_update(tx);
  }

  fn crash(&mut self, tx: &Sender<Self>, stderr: String) {
    self.status = LaunchStatus::Crashed;
    self.finish = Some(OffsetDateTime::now_utc());
    self.stderr = Some(stderr);

    self.send_update(tx);
  }

  fn send_update(&self, tx: &Sender<Self>) {
    let _ = tx.send(self.clone());
  }
}

//- Public launch handle
#[derive(Clone)]
pub struct LaunchHandle {
  pub id: String,
  pub rx: Receiver<LaunchState>,
  kill: KillFlag,
}
impl LaunchHandle {
  pub fn kill(&self) {
    self.kill.kill();
  }
}

//- Private download manager
struct InnerLaunch {
  state: Arc<RwLock<LaunchState>>,
  tx: Sender<LaunchState>,
  kill: KillFlag,
}

#[derive(Clone)]
pub(crate) struct Launch(Arc<InnerLaunch>);
impl Launch {
  pub async fn new(id: impl Into<String>) -> (Self, LaunchHandle) {
    let id = id.into();

    let launch = LaunchState::new(id.clone());
    let state = Arc::new(RwLock::new(launch));
    let (tx, rx) = tokio::sync::watch::channel(state.read().await.clone());

    let kill = KillFlag::default();
    let handle = LaunchHandle {
      id,
      rx,
      kill: kill.clone(),
    };

    let launch = Arc::new(InnerLaunch { state, tx, kill });
    launch.state.write().await.send_update(&launch.tx);
    (Self(launch), handle)
  }

  pub async fn start(&self) {
    self.0.state.write().await.start(&self.0.tx);
  }

  pub async fn set_playing(&self) {
    self.0.state.write().await.set_playing(&self.0.tx);
  }

  pub async fn fail(&self, error: String) {
    self.0.state.write().await.fail(&self.0.tx, error);
  }

  pub async fn crash(&self, stderr: String) {
    self.0.state.write().await.crash(&self.0.tx, stderr);
  }

  pub async fn exit(&self) {
    self.0.state.write().await.exit(&self.0.tx);
  }

  //

  pub fn is_killed(&self) -> bool {
    self.0.kill.is_killed()
  }
}

//- Process kill flag
#[derive(Default, Clone)]
struct KillFlag(Arc<AtomicBool>);
impl KillFlag {
  fn kill(&self) {
    self.0.store(true, Ordering::Relaxed);
  }

  fn is_killed(&self) -> bool {
    self.0.load(Ordering::Relaxed)
  }
}
