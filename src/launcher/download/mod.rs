mod models;
pub use models::*;

use crate::{
  managers::{AssetManager, ClientManager, LibraryManager, VersionManager},
  meta::{Asset, AssetIndex, ProcessedLibrary, VersionMeta},
};
use futures::future::try_join_all;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore, watch::Sender};

type Task<T> = tokio::task::JoinHandle<(T, crate::Result<Option<u64>>)>;

#[allow(clippy::too_many_arguments)]
pub fn download_game(
  version_id: &str,
  net_s: Arc<Semaphore>,
  io_s: Arc<Semaphore>,
  version_m: Arc<RwLock<VersionManager>>,
  lib_m: Arc<LibraryManager>,
  client_m: Arc<ClientManager>,
  asset_m: Arc<AssetManager>,
  state: Arc<RwLock<Download>>,
  tx: Sender<Download>,
) {
  let version_id = version_id.to_string();
  tokio::spawn(async move {
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let download_result: crate::Result<()> = async {
      // Note: Any error returned here will immediately stop the download with a failure

      // ! Download starting
      let meta = match version_m.read().await.meta(&version_id).await? {
        VersionMeta::Full(full_meta) => full_meta.process(), // Rule processing
        VersionMeta::Partial(_partial_meta) => {
          //todo
          //? find parent version & merge
          return Err(crate::Error::Download("Tried to download a partial metadata.".into()));
        }
      };
      let asset_index = asset_m.get_index(&meta.asset_index).await?;

      let libraries_size = lib_m.get_download_size(&meta.libraries).await;
      let assets_size = asset_m.get_download_size(&asset_index).await;

      // ! Download started
      let mut download = state.write().await;
      download.set_message(format!("{}.jar", &version_id), &tx);
      download.set_total(libraries_size.saturating_add(assets_size), &tx);
      download.start(&tx);

      //= CLIENT
      let client_size = client_m.download(&meta.downloads.client, &meta.id).await?;
      download.add_bytes(client_size, &tx);
      drop(download);

      //= LIBRARIES
      let mut existing_libs = Vec::new();
      let mut new_libs = Vec::new();
      let mut ignored_libs = Vec::new();
      let mut must_extract = Vec::new();

      let (lib_tasks, _skipped_libs) = build_lib_tasks(meta.libraries, &net_s, &io_s, &lib_m, &state, &tx);

      let lib_results = try_join_all(lib_tasks)
        .await
        .map_err(|e| crate::Error::Download(e.to_string()))?;

      for (lib, result) in lib_results {
        match result {
          Ok(Some(0)) => ignored_libs.push(lib), //? Impossible to download (no url).
          Ok(Some(_lib_size)) => {
            if lib.is_native {
              must_extract.push(lib);
            } else {
              new_libs.push(lib);
            }
          }
          Ok(None) => existing_libs.push(lib),
          Err(e) => {
            return Err(crate::Error::Download(format!(
              "Couldn't download library {}: {}",
              lib.name, e
            )));
          }
        }
      }

      //= ASSETS
      let mut new_assets = Vec::new();
      let mut existing_assets = Vec::new();

      let asset_tasks = build_asset_tasks(asset_index, &net_s, &io_s, &asset_m, &state, &tx);

      let asset_results = try_join_all(asset_tasks)
        .await
        .map_err(|e| crate::Error::Download(e.to_string()))?;

      for (asset, result) in asset_results {
        match result {
          Ok(Some(_asset_size)) => new_assets.push(asset),
          Ok(None) => existing_assets.push(asset),
          Err(e) => {
            return Err(crate::Error::Download(format!(
              "Couldn't download asset {}: {}",
              asset.hash, e
            )));
          }
        }
      }

      //= NATIVES EXTRACT
      let mut download = state.write().await;
      download.set_installing(&tx);
      for native_lib in &must_extract {
        download.set_message(&native_lib.path, &tx);

        lib_m.extract_native(native_lib).await?;
      }

      new_libs.extend(must_extract);

      Ok(())
    }
    .await;

    let mut download = state.write().await;

    match download_result {
      Ok(_) => download.finish("Download completed", &tx),
      Err(e) => download.fail(e.to_string(), &tx),
    }
  });
}

fn build_lib_tasks(
  libraries: Vec<ProcessedLibrary>,
  net_s: &Arc<Semaphore>,
  io_s: &Arc<Semaphore>,
  lib_m: &Arc<LibraryManager>,
  state: &Arc<RwLock<Download>>,
  tx: &Sender<Download>,
) -> (Vec<Task<ProcessedLibrary>>, Vec<ProcessedLibrary>) {
  let mut skipped_libs = Vec::new();
  let mut lib_tasks = Vec::new();

  for lib in libraries {
    if lib.must_skip {
      skipped_libs.push(lib);
      continue;
    }

    //= download task
    let net_s = Arc::clone(net_s);
    let io_s = Arc::clone(io_s);
    let state = Arc::clone(state);
    let tx = tx.clone();
    let lib_manager = Arc::clone(lib_m);

    let task = tokio::spawn(async move {
      state.write().await.set_message(&lib.path, &tx);

      let result: crate::Result<Option<u64>> = async {
        let lib_path = lib_manager.get_lib_path(&lib);
        if lib_path.exists() {
          return Ok(None);
        };

        let lib_url = match &lib.url {
          Some(url) => url,
          None => {
            // ! Library impossible to download.
            // ! Probably belongs to an external
            // ! version or contains malformed metadata
            return Ok(Some(0));
          }
        };

        let net_permit = net_s.acquire_owned().await.unwrap();
        state.write().await.set_message(&lib.path, &tx);
        let asset_bytes = crate::net::get_bytes(lib_url).await?;
        drop(net_permit);

        let io_permit = io_s.acquire_owned().await.unwrap();
        state.write().await.set_message(&lib.path, &tx);
        let lib_size = crate::io::write_bytes(&lib_path, &asset_bytes).await?;
        drop(io_permit);

        Ok(Some(lib_size))
      }
      .await;

      if let Ok(Some(lib_size)) = result {
        state.write().await.add_bytes(lib_size, &tx);
      }

      (lib, result)
    });

    lib_tasks.push(task);
  }

  (lib_tasks, skipped_libs)
}

fn build_asset_tasks(
  index: AssetIndex,
  net_s: &Arc<Semaphore>,
  io_s: &Arc<Semaphore>,
  asset_m: &Arc<AssetManager>,
  state: &Arc<RwLock<Download>>,
  tx: &Sender<Download>,
) -> Vec<Task<Asset>> {
  let mut asset_tasks = Vec::new();

  for asset in index.objects {
    //= download task
    let net_s = Arc::clone(net_s);
    let io_s = Arc::clone(io_s);
    let state = Arc::clone(state);
    let tx = tx.clone();
    let asset_manager = Arc::clone(asset_m);

    let task = tokio::spawn(async move {
      let result: crate::Result<Option<u64>> = async {
        let (asset_url, asset_path) = asset_manager.get_asset_url_and_path(&asset)?;
        if asset_path.exists() {
          return Ok(None);
        };

        let net_permit = net_s.acquire_owned().await.unwrap();
        state.write().await.set_message(&asset.path, &tx);
        let asset_bytes = crate::net::get_bytes(&asset_url).await?;
        drop(net_permit);

        let io_permit = io_s.acquire_owned().await.unwrap();
        state.write().await.set_message(&asset.path, &tx);
        let asset_size = crate::io::write_bytes(&asset_path, &asset_bytes).await?;
        drop(io_permit);

        Ok(Some(asset_size))
      }
      .await;

      if let Ok(Some(asset_size)) = result {
        state.write().await.add_bytes(asset_size, &tx);
      }

      (asset, result)
    });

    asset_tasks.push(task);
  }

  asset_tasks
}
