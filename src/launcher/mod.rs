mod builder;
pub use builder::{LauncherConfig, LauncherConfigBuilder};

mod managers;

mod download;
pub use download::*;
mod launch;
pub use launch::*;

use managers::{AssetManager, ClientManager, JavaManager, LibraryManager, VersionManager};
use std::{
  path::PathBuf,
  sync::{Arc, atomic::AtomicBool},
};
use tokio::sync::{RwLock, Semaphore, watch};
use url::Url;

pub(crate) type Instance = std::sync::Arc<InnerLauncher>;
pub(crate) struct InnerLauncher {
  pub launcher_name: String,
  pub launcher_version: String,
  pub root_dir: PathBuf,
  games_dir: PathBuf,

  pub version: RwLock<VersionManager>,
  pub library: LibraryManager,
  pub client: ClientManager,
  pub asset: AssetManager,
  pub java: RwLock<JavaManager>,

  net_s: Arc<Semaphore>,
  io_s: Arc<Semaphore>,
}

impl InnerLauncher {
  pub fn new(c: LauncherConfig) -> crate::Result<Self> {
    let games_dir = c.games_dir.unwrap_or_else(|| c.root_dir.join("instances"));
    let libraries_dir = c.libraries_dir.unwrap_or_else(|| c.root_dir.join("libraries"));
    let natives_dir = c.natives_dir.unwrap_or_else(|| c.root_dir.join("natives"));
    let versions_dir = c.versions_dir.unwrap_or_else(|| c.root_dir.join("versions"));
    let assets_dir = c.assets_dir.unwrap_or_else(|| c.root_dir.join("assets"));
    let java_dir = c.java_dir.unwrap_or_else(|| c.root_dir.join("java"));

    let manifest_url =
      Url::parse(&c.manifest_url).map_err(|e| crate::Error::Build(format!("Invalid manifest URL: {}", e)))?;
    let assets_url =
      Url::parse(&c.assets_url).map_err(|e| crate::Error::Build(format!("Invalid assets URL: {}", e)))?;

    let version = RwLock::new(VersionManager::new(
      versions_dir.clone(),
      manifest_url,
      c.manifest_path,
      c.manifest_refresh,
    ));

    let library = LibraryManager::new(libraries_dir, natives_dir);
    let client = ClientManager::new(versions_dir);
    let asset = AssetManager::new(assets_dir, assets_url);
    let java = RwLock::new(JavaManager::new(java_dir));

    let net_s = c.net_s.unwrap_or_else(|| Arc::new(Semaphore::new(10)));
    let io_s = c.io_s.unwrap_or_else(|| Arc::new(Semaphore::new(2)));

    Ok(Self {
      launcher_name: c.name,
      launcher_version: c.version,
      root_dir: c.root_dir,
      games_dir,
      //
      version,
      library,
      client,
      asset,
      java,
      //
      net_s,
      io_s,
    })
  }

  pub async fn init(&self) -> crate::Result<()> {
    let tree = [
      &self.root_dir,
      &self.games_dir,
      &self.version.read().await.versions_dir.clone(),
      &self.library.libraries_dir,
      &self.library.natives_dir,
      &self.asset.assets_dir,
      &self.java.read().await.java_dir,
    ];
    for folder_path in tree {
      crate::io::create_dir_all(folder_path).await?;
    }

    self.version.write().await.load_versions().await
  }
  //

  pub async fn download(
    &self,
    instance: Instance,
    download_id: impl Into<String>,
    version_id: impl Into<String>,
  ) -> DownloadHandle {
    let id: String = download_id.into();
    let download_state = Download::new(id.clone());

    let (tx, rx) = watch::channel(download_state.read().await.clone());
    let cancel_flag = Arc::new(AtomicBool::new(false));

    let version_id = version_id.into();
    let state = Arc::clone(&download_state);
    let cancel = Arc::clone(&cancel_flag);
    download::download_game(instance, version_id, state, tx, cancel);

    DownloadHandle::new(id, rx, cancel_flag)
  }

  pub async fn launch(&self, instance: Instance, config: LaunchConfig) -> crate::Result<LaunchHandle> {
    launch::launch_game(instance, config).await
  }
}
