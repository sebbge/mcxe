use shared::*;

mod platform;

mod error;
pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

mod launcher;
mod meta;
pub use launcher::*;

#[derive(Clone)]
pub struct Launcher(Instance);
impl Launcher {
  pub fn new(config: LauncherConfig) -> Self {
    Self(std::sync::Arc::new(InnerLauncher::new(config)))
  }

  pub async fn init(&self) -> Result<()> {
    self.0.init().await
  }

  pub async fn refresh_versions(&self) -> Result<()> {
    self.0.version.write().await.init().await
  }
  //

  pub async fn download(
    &self,
    download_id: impl Into<String>,
    version_id: &str,
  ) -> crate::Result<(Option<DownloadHandle>, DownloadHandle)> {
    let instance = std::sync::Arc::clone(&self.0);
    self.0.download(instance, download_id.into(), version_id).await
  }

  pub async fn launch(&self, config: LaunchConfig) -> crate::Result<LaunchHandle> {
    let instance = std::sync::Arc::clone(&self.0);
    self.0.launch(instance, config).await
  }
}
