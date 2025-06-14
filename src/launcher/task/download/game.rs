use super::{super::TaskManager, Download, DownloadJob, DownloadResult};
use crate::Instance;
use crate::meta::{Asset, AssetIndex, File, ProcessedLibrary, ProcessedVersionMeta};
use futures::future;

pub(crate) fn download_game(instance: Instance, download: TaskManager<Download>, meta: ProcessedVersionMeta) {
  tokio::spawn(async move {
    let download_result: crate::Result<Option<()>> = async {
      //- Meta
      let libraries = meta.libraries;
      let asset_index = instance.asset.get_index(&meta.asset_index).await?;
      let client = meta.downloads.client;

      let libraries_size = instance.library.get_download_size(&libraries).await;
      let assets_size = instance.asset.get_download_size(&asset_index).await;
      let client_size = instance.client.get_download_size(&client, &meta.id).await;
      let total = crate::util::op::saturating_sum(&[libraries_size, assets_size, client_size]);
      download.set_total(total).await;
      download.start().await;

      //- Libraries download
      let (lib_tasks, _skipped_libs) = build_lib_tasks(&instance, &download, libraries);
      let lib_results = future::try_join_all(lib_tasks)
        .await
        .map_err(|e| crate::Error::Download(e.to_string()))?;

      let mut existing_libs = Vec::new();
      let mut new_libs = Vec::new();
      let mut ignored_libs = Vec::new();
      let mut must_extract = Vec::new();
      for (lib, result) in lib_results {
        match result {
          Ok(DownloadResult::Cancelled) => return Ok(None),
          Ok(DownloadResult::Success) => {
            if lib.is_native {
              must_extract.push(lib);
            } else {
              new_libs.push(lib);
            }
          }
          Ok(DownloadResult::Ignored) => existing_libs.push(lib),
          Ok(DownloadResult::Impossible) => ignored_libs.push(lib), //? Impossible to download (no url).
          Err(e) => {
            return Err(crate::Error::Download(format!(
              "Couldn't download library {}: {}",
              lib.name, e
            )));
          }
        }
      }

      //- Assets download
      let asset_tasks = build_asset_tasks(&instance, &download, asset_index);
      let asset_results = future::try_join_all(asset_tasks)
        .await
        .map_err(|e| crate::Error::Download(e.to_string()))?;

      let mut new_assets = Vec::new();
      let mut existing_assets = Vec::new();
      for (asset, result) in asset_results {
        match result {
          Ok(DownloadResult::Cancelled) => return Ok(None),
          Ok(DownloadResult::Success) => new_assets.push(asset),
          Ok(DownloadResult::Ignored) => existing_assets.push(asset),
          Ok(DownloadResult::Impossible) => (),
          Err(e) => {
            return Err(crate::Error::Download(format!(
              "Couldn't download asset {}: {}",
              asset.hash, e
            )));
          }
        }
      }

      //- Client download
      let client_task = build_client_task(&instance, &download, client, &meta.id);
      let (_client, result) = client_task.await.map_err(|e| crate::Error::Download(e.to_string()))?;
      match result {
        Ok(DownloadResult::Cancelled) => return Ok(None),
        Ok(DownloadResult::Success) => (),
        Ok(DownloadResult::Ignored) => (),
        Ok(DownloadResult::Impossible) => (),
        Err(e) => {
          return Err(crate::Error::Download(format!(
            "Couldn't download game client for {}: {}",
            &meta.id, e
          )));
        }
      };

      Ok(Some(()))
    }
    .await;

    match download_result {
      Ok(None) => download.abort().await,
      Ok(Some(())) => download.success().await,
      Err(e) => download.fail(e.to_string()).await,
    };
  });
}

fn build_lib_tasks(
  instance: &Instance,
  download: &TaskManager<Download>,
  libraries: Vec<ProcessedLibrary>,
) -> (Vec<DownloadJob<ProcessedLibrary>>, Vec<ProcessedLibrary>) {
  let mut skipped = Vec::new();
  let mut tasks = Vec::new();

  for lib in libraries {
    if lib.must_skip {
      skipped.push(lib);
      continue;
    }

    let instance = instance.clone();
    let download = download.clone();
    tasks.push(tokio::spawn(async move {
      let result = async {
        if download.was_aborted() {
          return Ok(DownloadResult::Cancelled);
        }

        let path = instance.library.get_lib_path(&lib);
        if path.exists() {
          return Ok(DownloadResult::Ignored);
        };

        let url = match &lib.url {
          Some(url) => url,
          None => {
            // ! Library impossible to download; probably belongs to
            // ! an external version or contains malformed metadata
            return Ok(DownloadResult::Impossible);
          }
        };

        let net_permit = instance.net_s.clone().acquire_owned().await.unwrap();
        download.set_message(&lib.path).await;
        let bytes = crate::net::get_bytes(url).await?;
        drop(net_permit);

        let io_permit = instance.io_s.clone().acquire_owned().await.unwrap();
        // download.set_message(&lib.path);
        crate::io::write(&path, &bytes).await?;
        drop(io_permit);

        download.add_bytes(bytes.len(), &lib.path).await;

        Ok(DownloadResult::Success)
      }
      .await;

      (lib, result)
    }));
  }

  (tasks, skipped)
}

fn build_asset_tasks(
  instance: &Instance,
  download: &TaskManager<Download>,
  index: AssetIndex,
) -> Vec<DownloadJob<Asset>> {
  let mut tasks = Vec::new();

  for asset in index.objects {
    let instance = instance.clone();
    let download = download.clone();
    tasks.push(tokio::spawn(async move {
      let result = async {
        if download.was_aborted() {
          return Ok(DownloadResult::Cancelled);
        }

        let (url, path) = instance.asset.get_asset_url_and_path(&asset)?;
        if path.exists() {
          return Ok(DownloadResult::Ignored);
        };

        let net_permit = instance.net_s.clone().acquire_owned().await.unwrap();
        download.set_message(&asset.path).await;
        let bytes = crate::net::get_bytes(&url).await?;
        drop(net_permit);

        let io_permit = instance.io_s.clone().acquire_owned().await.unwrap();
        // download.await.set_message(&asset.path);
        crate::io::write(&path, &bytes).await?;
        drop(io_permit);

        download.add_bytes(bytes.len(), &asset.path).await;

        Ok(DownloadResult::Success)
      }
      .await;

      (asset, result)
    }));
  }

  tasks
}

fn build_client_task(
  instance: &Instance,
  download: &TaskManager<Download>,
  client: File,
  v_id: &str,
) -> DownloadJob<File> {
  let v_id = v_id.to_string();
  let instance = instance.clone();
  let download = download.clone();
  tokio::spawn(async move {
    let result = async {
      if download.was_aborted() {
        return Ok(DownloadResult::Cancelled);
      }

      let path = instance.client.get_path(&v_id);
      if path.exists() {
        return Ok(DownloadResult::Ignored);
      };
      let message = format!("{}.jar", &v_id);

      let net_permit = instance.net_s.clone().acquire_owned().await.unwrap();
      download.set_message(&message).await;
      let bytes = crate::net::get_bytes(&client.url).await?;
      drop(net_permit);

      let io_permit = instance.io_s.clone().acquire_owned().await.unwrap();
      // download.set_message(&message);
      crate::io::write(&path, &bytes).await?;
      drop(io_permit);

      download.add_bytes(bytes.len(), &message).await;

      Ok(DownloadResult::Success)
    }
    .await;

    (client, result)
  })
}
