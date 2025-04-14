mod builder;
pub use builder::LauncherBuilder;

pub mod download;
pub mod java;
pub mod meta;

use download::Download;
use java::JavaManager;
use meta::{AssetManager, ClientManager, LibraryManager, VersionManager, models::version::VersionMeta};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::{
  RwLock,
  watch::{self, Receiver},
};

pub struct Launcher {
  launcher_name: String,
  launcher_version: String,
  pub root_dir: PathBuf,
  games_dir: PathBuf,
  natives_dir: PathBuf,

  pub libraries: Arc<LibraryManager>,
  pub versions: Arc<RwLock<VersionManager>>,
  pub client: Arc<ClientManager>,
  pub assets: Arc<AssetManager>,
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
      &self.java.read().await.java_dir,
      &self.libraries.libraries_dir,
      &self.assets.assets_dir,
      &self.natives_dir,
      &self.versions.read().await.versions_dir,
    ];

    for folder_path in tree {
      crate::io::create_dir_all(folder_path).await?;
    }

    self.refresh_versions().await?;

    Ok(())
  }

  pub async fn refresh_versions(&self) -> crate::Result<()> {
    self.versions.write().await.load_versions().await
  }

  //

  pub async fn download(&self, download_id: impl Into<String>, version_id: &str) -> crate::Result<Receiver<Download>> {
    let download_state = Arc::new(RwLock::new(Download::new(download_id)));

    let (tx, rx) = watch::channel(download_state.read().await.clone());
    let _ = tx.send(download_state.read().await.clone()); //?? already sends when channel create?

    // let libraries = Arc::clone(&self.libraries);
    let versions = Arc::clone(&self.versions);
    let client = Arc::clone(&self.client);
    let assets = Arc::clone(&self.assets);
    // let java = Arc::clone(&self.java);
    let state = Arc::clone(&download_state);
    let v_id = version_id.to_string();

    tokio::spawn(async move {
      tokio::time::sleep(std::time::Duration::from_millis(50)).await;

      let download_result: crate::Result<()> = async {
        let mut download = state.write().await;

        let meta = match versions.read().await.meta(&v_id).await? {
          VersionMeta::Full(full_meta) => full_meta,
          VersionMeta::Partial(_partial_meta) => {
            return Err(crate::Error::Download("Tried to download a partial metadata.".into()));
          }
        };

        let assets = assets.get_index(&meta.asset_index).await?;

        let client_size = client.download(&meta.downloads.client, &meta.id).await?;
        download.update_progress(client_size);
        download.send_update(&tx);

        // for lib in meta.libraries {
        //   total_bytes_downloaded += self.libraries.download_one(&lib).await?;
        // }
        Ok(())
      }
      .await;

      match download_result {
        Ok(()) => {}
        Err(e) => {
          let mut download = state.write().await;
          download.fail(e.to_string());
          download.send_update(&tx);
        }
      }
    });

    Ok(rx)
  }

  pub async fn launch(&self) {
    println!("Launching...")
  }
}
