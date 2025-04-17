mod builder;
pub use builder::LauncherBuilder;

pub mod managers;
pub mod meta;

mod download;
use download::*;
mod launch;
// use launch::*;

use managers::{AssetManager, ClientManager, JavaManager, LibraryManager, VersionManager};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::{
  RwLock, Semaphore,
  watch::{self, Receiver},
};

pub struct Launcher {
  launcher_name: String,
  launcher_version: String,
  pub root_dir: PathBuf,
  games_dir: PathBuf,

  pub version: Arc<RwLock<VersionManager>>,
  pub library: Arc<LibraryManager>,
  pub client: Arc<ClientManager>,
  pub asset: Arc<AssetManager>,
  pub java: Arc<RwLock<JavaManager>>,
}

impl Launcher {
  pub fn builder() -> LauncherBuilder {
    LauncherBuilder::default()
  }

  pub async fn init(&mut self) -> crate::Result<()> {
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

  pub async fn refresh_versions(&self) -> crate::Result<()> {
    self.version.write().await.load_versions().await
  }

  //

  pub async fn download(&self, download_id: impl Into<String>, version_id: &str) -> Receiver<Download> {
    let net_s = Arc::new(Semaphore::new(16));
    let io_s = Arc::new(Semaphore::new(2));

    let download_state = Arc::new(RwLock::new(Download::new(download_id)));
    let (tx, rx) = watch::channel(download_state.read().await.clone());

    // let java = Arc::clone(&self.java);

    download::download_game(
      version_id,
      Arc::clone(&net_s),
      Arc::clone(&io_s),
      Arc::clone(&self.version),
      Arc::clone(&self.library),
      Arc::clone(&self.client),
      Arc::clone(&self.asset),
      Arc::clone(&download_state),
      tx,
    );

    rx
  }

  pub async fn launch(&self) {
    println!("Launching...")
  }
}
