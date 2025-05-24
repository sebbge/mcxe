mod models;
pub use models::*;

mod command;

use crate::{Instance, meta::ProcessedVersionMeta};
use std::process::Stdio;
use tokio::process::Child;

use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::RwLock;

pub(super) async fn launch_game(instance: Instance, launch: Launch, meta: ProcessedVersionMeta, config: LaunchConfig) {
  let mut command = match command::build(instance, &launch, meta, &config).await {
    Ok(cmd) => cmd,
    Err(e) => {
      tracing::error!("{}", e);
      launch.fail(e.to_string()).await;
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
      launch.fail(e.to_string()).await;
      return;
    }
  };

  tracing::info!("Minecraft is running");

  tokio::spawn(watch_process(launch, child));
}

async fn watch_process(launch: Launch, mut child: Child) {
  let stdout = child.stdout.take();
  let stderr = child.stderr.take();

  let stdout_log = Arc::new(RwLock::new(String::new()));
  let stderr_log = Arc::new(RwLock::new(String::new()));

  if let Some(stdout) = stdout {
    let log_clone = stdout_log.clone();
    tokio::spawn(async move {
      let reader = BufReader::new(stdout);
      let mut lines = reader.lines();
      while let Ok(Some(line)) = lines.next_line().await {
        log_clone.write().await.push_str(&format!("{line}\n"));
      }
    });
  }

  if let Some(stderr) = stderr {
    let log_clone = stderr_log.clone();
    tokio::spawn(async move {
      let reader = BufReader::new(stderr);
      let mut lines = reader.lines();
      while let Ok(Some(line)) = lines.next_line().await {
        log_clone.write().await.push_str(&format!("{line}\n"));
      }
    });
  }

  loop {
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    launch.set_playing().await;

    if launch.is_killed() {
      if let Err(e) = child.kill().await {
        tracing::warn!("Failed to kill process: {e}");
        break;
      }
      tracing::info!("Minecraft exited successfully!");
      launch.exit().await;
      break;
    }

    match child.try_wait() {
      Ok(None) => continue,
      Ok(Some(output)) => {
        if output.success() {
          tracing::info!("Minecraft exited successfully!");
          launch.exit().await;
          tracing::info!("OUT: {}", stdout_log.read().await.clone());
        } else {
          tracing::error!("Minecraft exited with an error.");

          launch.crash(stderr_log.read().await.clone()).await;
          tracing::error!("ERR: {}", stderr_log.read().await.clone());
        }
        break;
      }
      Err(e) => {
        tracing::error!("Error while waiting for Minecraft process: {e}");
        launch.fail(e.to_string()).await;
        break;
      }
    }
  }
}
