use crate::{DownloadHandle, LaunchHandle, ProgressHandle, Result};

mod builder;
pub use builder::{LauncherConfig, LauncherConfigBuilder};

mod task;
pub use task::*;

mod managers;
use managers::*;

use std::{path::PathBuf, sync::Arc};
use tokio::sync::{RwLock, Semaphore};

pub(crate) type Instance = Arc<InnerLauncher>;
pub(crate) struct InnerLauncher {
  launcher_name: String,
  launcher_version: String,
  pub root_dir: PathBuf,
  pub instances_dir: PathBuf,
  pub version: RwLock<VersionManager>,
  library: LibraryManager,
  asset: AssetManager,
  java: JavaManager,
  net_s: Arc<Semaphore>,
  io_s: Arc<Semaphore>,
}
impl InnerLauncher {
  pub fn new(c: LauncherConfig) -> Self {
    let instances_dir = c.root_dir.join("instances");
    let libraries_dir = c.root_dir.join("libraries");
    let natives_dir = c.root_dir.join("natives");
    let versions_dir = c.root_dir.join("versions");
    let assets_dir = c.root_dir.join("assets");
    let java_dir = c.root_dir.join("java");

    let manifest_url = url::Url::parse(&c.manifest_url).unwrap_or_else(|e| panic!("Invalid manifest URL: {}", e));
    let assets_url = url::Url::parse(&c.assets_url).unwrap_or_else(|e| panic!("Invalid assets URL: {}", e));

    let version = RwLock::new(VersionManager::new(versions_dir.clone(), manifest_url));

    let library = LibraryManager::new(libraries_dir, natives_dir, versions_dir);
    let asset = AssetManager::new(assets_dir, assets_url);
    let java = JavaManager::new(java_dir);

    Self {
      launcher_name: c.name,
      launcher_version: c.version,
      root_dir: c.root_dir,
      instances_dir,
      version,
      library,
      asset,
      java,
      net_s: c.net_s.unwrap_or_else(|| Arc::new(Semaphore::new(10))),
      io_s: c.io_s.unwrap_or_else(|| Arc::new(Semaphore::new(4))),
    }
  }

  pub async fn init(&self) -> Result<()> {
    for dir in [
      &self.root_dir,
      &self.instances_dir,
      &self.version.read().await.versions_dir.clone(),
      &self.library.libraries_dir,
      &self.library.natives_dir,
      &self.asset.assets_dir,
      &self.java.java_dir,
    ] {
      crate::fs::create_dir_all(dir).await?;
    }

    self.version.write().await.init().await
  }
  //

  pub async fn download(
    &self,
    instance: Instance,
    version_id: &str,
  ) -> Result<(Option<DownloadHandle>, DownloadHandle)> {
    let meta = instance.version.read().await.meta(version_id).await?;

    //= JAVA
    let java_handle = match instance.java.get_java(&meta.java_version) {
      Ok(_) => None,
      Err(_) => {
        let java_version = meta.java_version.as_ref().map(|j| j.major_version).unwrap_or(21);
        let id = format!("jre-{}:{}", java_version, &version_id);

        let (download, handle) = DownloadTask::new(id).await;
        download_java(Arc::clone(&instance), download, java_version);
        Some(handle)
      }
    };

    //= GAME
    let (download, mc_handle) = DownloadTask::new(version_id).await;
    download_game(instance, download, meta);
    Ok((java_handle, mc_handle))
  }

  pub async fn launch(&self, instance: Instance, config: LaunchConfig) -> Result<LaunchHandle> {
    let meta = instance.version.read().await.meta(&config.version).await?;

    let (launch, handle) = LaunchTask::new(&config.id).await;
    launch_game(instance, launch, meta, config).await;
    Ok(handle)
  }

  pub async fn checksum(&self, instance: Instance, version_id: &str) -> Result<ProgressHandle> {
    let meta = instance.version.read().await.meta(version_id).await?;

    let (progress, handle) = ProgressTask::new(version_id).await;
    checksum(instance, progress, meta, false);
    Ok(handle)
  }
}
impl Default for InnerLauncher {
  fn default() -> Self {
    Self::new(LauncherConfig::default())
  }
}
