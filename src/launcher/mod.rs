mod builder;
pub use builder::{LauncherConfig, LauncherConfigBuilder};

mod managers;

mod download;
pub use download::*;
mod launch;
pub use launch::*;

use managers::{AssetManager, ClientManager, JavaManager, LibraryManager, VersionManager};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};

pub(crate) type Instance = std::sync::Arc<InnerLauncher>;
pub(crate) struct InnerLauncher {
  launcher_name: String,
  launcher_version: String,
  pub root_dir: PathBuf,
  pub games_dir: PathBuf,

  pub version: RwLock<VersionManager>,
  library: LibraryManager,
  client: ClientManager,
  asset: AssetManager,
  java: JavaManager,

  net_s: Arc<Semaphore>,
  io_s: Arc<Semaphore>,
}

impl InnerLauncher {
  pub fn new(c: LauncherConfig) -> Self {
    let games_dir = c.games_dir.unwrap_or_else(|| c.root_dir.join("instances"));
    let libraries_dir = c.libraries_dir.unwrap_or_else(|| c.root_dir.join("libraries"));
    let natives_dir = c.natives_dir.unwrap_or_else(|| c.root_dir.join("natives"));
    let versions_dir = c.versions_dir.unwrap_or_else(|| c.root_dir.join("versions"));
    let assets_dir = c.assets_dir.unwrap_or_else(|| c.root_dir.join("assets"));
    let java_dir = c.java_dir.unwrap_or_else(|| c.root_dir.join("java"));

    let manifest_url = url::Url::parse(&c.manifest_url).unwrap_or_else(|e| panic!("Invalid manifest URL: {}", e));
    let assets_url = url::Url::parse(&c.assets_url).unwrap_or_else(|e| panic!("Invalid manifest URL: {}", e));

    let version = RwLock::new(VersionManager::new(
      versions_dir.clone(),
      manifest_url,
      c.manifest_path,
      c.manifest_refresh,
    ));

    let library = LibraryManager::new(libraries_dir, natives_dir);
    let client = ClientManager::new(versions_dir);
    let asset = AssetManager::new(assets_dir, assets_url);
    let java = JavaManager::new(java_dir);

    let net_s = c.net_s.unwrap_or_else(|| Arc::new(Semaphore::new(10)));
    let io_s = c.io_s.unwrap_or_else(|| Arc::new(Semaphore::new(2)));

    Self {
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
    }
  }

  pub async fn init(&self) -> crate::Result<()> {
    let tree = [
      &self.root_dir,
      &self.games_dir,
      &self.version.read().await.versions_dir.clone(),
      &self.library.libraries_dir,
      &self.library.natives_dir,
      &self.asset.assets_dir,
      &self.java.java_dir,
    ];
    for folder_path in tree {
      crate::io::create_dir_all(folder_path).await?;
    }

    self.version.write().await.init().await
  }
  //

  pub async fn download(
    &self,
    instance: Instance,
    version_id: &str,
  ) -> crate::Result<(Option<DownloadHandle>, DownloadHandle)> {
    let meta = instance.version.read().await.meta(version_id).await?;

    //= JAVA
    let (path_to_java, java_version) = instance.java.get_java(&meta.java_version)?;
    let java_handle = if path_to_java.is_none() {
      let id = format!("jre-{}:{}", java_version, &version_id);

      let (download, handle) = Download::new(id).await;

      download::download_java(Arc::clone(&instance), download, java_version);

      Some(handle)
    } else {
      None
    };

    //= GAME
    let (download, mc_handle) = Download::new(version_id).await;
    download::download_game(instance, download, meta);
    Ok((java_handle, mc_handle))
  }

  pub async fn launch(&self, instance: Instance, config: LaunchConfig) -> crate::Result<LaunchHandle> {
    let meta = instance.version.read().await.meta(&config.version).await?;

    let (launch, handle) = Launch::new(&config.id).await;
    launch::launch_game(instance, launch, meta, config).await;
    Ok(handle)
  }
}
