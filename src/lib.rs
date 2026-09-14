mod fs;
pub mod net;
mod util;
pub use taskee::{TaskData, TaskHandle};

mod error;
pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

mod launcher;
pub use launcher::*;

mod meta;
pub use meta::{LatestVersions, Version, VersionStatus, VersionType};

use std::{path::Path, sync::Arc};

pub type DownloadHandle = TaskHandle<Download>;
pub type LaunchHandle = TaskHandle<Launch>;
pub type ProgressHandle = TaskHandle<Progress>;
pub type DownloadData = TaskData<Download>;
pub type ProcessData = TaskData<Launch>;
pub type ProgressData = TaskData<Progress>;

#[derive(Default, Clone)]
pub struct Launcher(Instance);
impl Launcher {
  pub fn new(config: LauncherConfig) -> Self {
    Self(Arc::new(InnerLauncher::new(config)))
  }

  pub async fn init(&self) -> Result<()> {
    self.0.init().await
  }

  pub fn root_dir(&self) -> &Path {
    &self.0.root_dir
  }

  pub fn instances_dir(&self) -> &Path {
    &self.0.instances_dir
  }

  pub async fn reload_versions(&self) -> Result<()> {
    self.0.version.write().await.init().await
  }

  pub async fn versions(&self) -> Vec<meta::Version> {
    self.0.version.read().await.versions()
  }

  pub async fn latest_versions(&self) -> LatestVersions {
    self.0.version.read().await.latest_versions().await
  }
  //

  pub async fn download(&self, version_id: &str) -> Result<(Option<DownloadHandle>, DownloadHandle)> {
    Arc::clone(&self.0).download(version_id).await
  }

  pub async fn launch(&self, config: LaunchConfig) -> Result<LaunchHandle> {
    Arc::clone(&self.0).launch(config).await
  }

  pub async fn checksum(&self, version_id: &str, check_assets: bool) -> Result<ProgressHandle> {
    Arc::clone(&self.0).checksum(version_id, check_assets).await
  }
}
