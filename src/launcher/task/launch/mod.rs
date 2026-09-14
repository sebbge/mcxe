mod command;

use crate::{Instance, meta::ProcessedVersionMeta};
use serde::Serialize;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, RwLock};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;

use taskee::{Task, TaskStatus};

const DEFAULT_UUID: &str = "00000000-0000-0000-0000-000000000000";

#[derive(bon::Builder)]
#[builder(on(_, into))]
pub struct LaunchConfig {
  pub(crate) id: String,
  pub(crate) version: String,
  pub(super) username: String,
  /// Defaults to the nil UUID, for offline play.
  #[builder(default = DEFAULT_UUID)]
  pub(super) uuid: String,
  pub(super) token: Option<String>,
  pub(super) directory: String,
  pub(super) custom_java: Option<PathBuf>,
  pub(super) extra_jvm_args: Option<Vec<String>>,
  pub(super) extra_game_args: Option<Vec<String>>,
}

// launch task
#[cfg_attr(feature = "ts-types", derive(ts_rs::TS), ts(export))]
#[derive(TaskStatus, Serialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum LaunchStatus {
  #[default]
  Pending,
  Starting,
  Running,
  #[terminal]
  Failed,
  #[terminal]
  Aborted,
  #[terminal]
  Success,
}
#[derive(Task, Default, Clone)]
#[taskee(status)]
pub struct Launch {
  pub status: LaunchStatus,
  pub stdout: Logs,
  pub stderr: Logs,
}

#[derive(Clone, Default, Debug)]
pub struct Logs(Arc<RwLock<Vec<String>>>);

impl Logs {
  pub(crate) fn push(&self, line: String) {
    self.0.write().unwrap().push(line);
  }

  pub fn len(&self) -> usize {
    self.0.read().unwrap().len()
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  pub fn since(&self, start: usize) -> Vec<String> {
    self.0.read().unwrap()[start..].to_vec()
  }

  pub fn lines(&self) -> Vec<String> {
    self.0.read().unwrap().clone()
  }
}

impl LaunchTask {
  pub(crate) fn add_line_stdout(&self, line: String) {
    self.set(|payload| payload.stdout.push(line));
  }

  pub(crate) fn add_line_stderr(&self, line: String) {
    self.set(|payload| payload.stderr.push(line));
  }
}

pub(crate) async fn launch_game(
  instance: Instance,
  process: LaunchTask,
  meta: ProcessedVersionMeta,
  config: LaunchConfig,
) {
  let mut command = match command::build(instance, &process, meta, &config).await {
    Ok(cmd) => cmd,
    Err(e) => {
      tracing::error!("{}", e);
      process.failed();
      return;
    }
  };

  let child = match command
    .stdout(Stdio::piped()) // mc logs
    .stderr(Stdio::piped())
    .spawn()
  {
    Ok(child) => child,
    Err(e) => {
      tracing::error!("failed to execute process: {}", e);
      process.failed();
      return;
    }
  };

  tracing::info!("Minecraft is running");

  tokio::spawn(watch_process(process, child));
}

async fn watch_process(process: LaunchTask, mut child: Child) {
  if let Some(stdout) = child.stdout.take() {
    let process = process.clone();

    tokio::spawn(async move {
      let mut lines = BufReader::new(stdout).lines();
      while let Ok(Some(line)) = lines.next_line().await {
        process.add_line_stdout(format!("{line}\n"));
      }
    });
  }

  if let Some(stderr) = child.stderr.take() {
    let process = process.clone();

    tokio::spawn(async move {
      let mut lines = BufReader::new(stderr).lines();
      while let Ok(Some(line)) = lines.next_line().await {
        process.add_line_stderr(format!("{line}\n"));
      }
    });
  }

  loop {
    let tick = tokio::time::sleep(tokio::time::Duration::from_secs(1));

    if process.until_aborted(tick).await.is_none() {
      if let Err(e) = child.kill().await {
        tracing::warn!("failed to kill process: {e}");
        break;
      }
      tracing::info!("game exited successfully");
      process.success();
      break;
    }

    process.set_running();

    match child.try_wait() {
      Ok(None) => continue,
      Ok(Some(output)) => {
        if output.success() {
          tracing::info!("game exited successfully");
          process.success();
        } else {
          tracing::error!("game exited with an error");
          process.aborted();
        }
        break;
      }
      Err(e) => {
        tracing::error!("error while waiting for process: {e}");
        process.failed();
        break;
      }
    }
  }
}
