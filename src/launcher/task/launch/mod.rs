mod models;
pub use models::*;

mod command;

use crate::{Instance, meta::ProcessedVersionMeta};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;

pub(crate) async fn launch_game(
  instance: Instance,
  process: ProcessTask,
  meta: ProcessedVersionMeta,
  config: LaunchConfig,
) {
  let mut command = match command::build(instance, &process, meta, &config).await {
    Ok(cmd) => cmd,
    Err(e) => {
      tracing::error!("{}", e);
      process.fail(e.to_string()).await;
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
      process.fail(e.to_string()).await;
      return;
    }
  };

  tracing::info!("Minecraft is running");

  tokio::spawn(watch_process(process, child));
}

async fn watch_process(process: ProcessTask, mut child: Child) {
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

    if process.is_killed() {
      if let Err(e) = child.kill().await {
        tracing::warn!("Failed to kill process: {e}");
        break;
      }
      tracing::info!("game exited successfully");
      process.exit().await;
      break;
    }

    match child.try_wait() {
      Ok(None) => continue,
      Ok(Some(output)) => {
        if output.success() {
          tracing::info!("game exited successfully");
          process.exit().await;
        } else {
          tracing::error!("game exited with an error");
          process.crash().await;
        }
        break;
      }
      Err(e) => {
        tracing::error!("Error while waiting for Minecraft process: {e}");
        process.fail(e.to_string()).await;
        break;
      }
    }
  }
}
