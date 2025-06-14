use super::super::{Task, TaskManager, TaskStatus};
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
#[derive(Default, Clone)]
pub struct Process {
  pub is_previous: bool,
  pub stdout: Vec<String>,
  pub stderr: Vec<String>,
}
impl Task<Process> {
  fn add_line_stdout(&mut self, tx: &Sender<Self>, line: String) {
    self.payload.stdout.push(line);

    self.send_update(tx);
  }

  fn add_line_stderr(&mut self, tx: &Sender<Self>, line: String) {
    self.payload.stderr.push(line);

    self.send_update(tx);
  }
}

//--- Private process manager
impl TaskManager<Process> {
  pub async fn add_line_stdout(&self, line: String) {
    self.0.state.write().await.add_line_stdout(&self.0.tx, line);
  }

  pub async fn add_line_stderr(&self, line: String) {
    self.0.state.write().await.add_line_stderr(&self.0.tx, line);
  }

  pub async fn set_playing(&self) {
    self.0.state.write().await.set_status(&self.0.tx, TaskStatus::Running);
  }
}
