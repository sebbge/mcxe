mod command;

use crate::{Instance, meta::ProcessedVersionMeta};
use serde::Serialize;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;

use tasked::{Task, TaskStatus};

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

// launch task
#[derive(TaskStatus, Serialize, Default, Clone, Debug)]
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
#[tasked(status)]
pub struct Launch {
  pub status: LaunchStatus,
  pub is_previous: bool,
  pub stdout: Vec<String>,
  pub stderr: Vec<String>,
}

// private launch manager
impl LaunchTask {
  pub async fn add_line_stdout(&self, line: String) {
    self
      .update(|payload| {
        payload.stdout.push(line);
        true
      })
      .await;
  }

  pub async fn add_line_stderr(&self, line: String) {
    self
      .update(|payload| {
        payload.stderr.push(line);
        true
      })
      .await;
  }

  pub async fn set_playing(&self) {
    self
      .update(|payload| {
        payload.status.set_running();
        true
      })
      .await;
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
      process.failed().await;
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
      tracing::error!("Failed to execute Minecraft process: {}", e);
      process.failed().await;
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
        process.add_line_stdout(format!("{line}\n")).await;
      }
    });
  }

  if let Some(stderr) = child.stderr.take() {
    let process = process.clone();

    tokio::spawn(async move {
      let mut lines = BufReader::new(stderr).lines();
      while let Ok(Some(line)) = lines.next_line().await {
        process.add_line_stderr(format!("{line}\n")).await;
      }
    });
  }

  loop {
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    process.set_playing().await;

    if process.was_aborted() {
      if let Err(e) = child.kill().await {
        tracing::warn!("Failed to kill process: {e}");
        break;
      }
      tracing::info!("game exited successfully");
      process.success().await;
      break;
    }

    match child.try_wait() {
      Ok(None) => continue,
      Ok(Some(output)) => {
        if output.success() {
          tracing::info!("game exited successfully");
          process.success().await;
        } else {
          tracing::error!("game exited with an error");
          process.aborted().await;
        }
        break;
      }
      Err(e) => {
        tracing::error!("Error while waiting for Minecraft process: {e}");
        process.failed().await;
        break;
      }
    }
  }
}
