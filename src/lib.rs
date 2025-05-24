use std::path::Path;

use shared::*;

mod platform;

mod error;
pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

mod launcher;
mod meta;
pub use launcher::*;

pub use meta::{LatestVersions, Version, VersionStatus, VersionType};

#[derive(Clone)]
pub struct Launcher(Instance);
impl Launcher {
  pub fn new(config: LauncherConfig) -> Self {
    Self(std::sync::Arc::new(InnerLauncher::new(config)))
  }

  pub async fn init(&self) -> Result<()> {
    self.0.init().await
  }

  pub fn root_dir(&self) -> &Path {
    &self.0.root_dir
  }

  pub fn games_dir(&self) -> &Path {
    &self.0.games_dir
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

  pub async fn download(&self, version_id: &str) -> crate::Result<(Option<DownloadHandle>, DownloadHandle)> {
    let instance = std::sync::Arc::clone(&self.0);
    self.0.download(instance, version_id).await
  }

  pub async fn launch(&self, config: LaunchConfig) -> crate::Result<LaunchHandle> {
    let instance = std::sync::Arc::clone(&self.0);
    self.0.launch(instance, config).await
  }
}
