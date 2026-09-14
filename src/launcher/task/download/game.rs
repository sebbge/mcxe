use super::{DownloadJob, DownloadResult, DownloadStatusExt, DownloadTask};
use crate::Instance;
use crate::meta::{Asset, AssetIndex, File, ProcessedLibrary, ProcessedVersionMeta};
use futures::FutureExt;
use futures::future;

pub(crate) fn download_game(instance: Instance, download: DownloadTask, meta: ProcessedVersionMeta) {
  tokio::spawn(async move {
    let download_result: crate::Result<Option<()>> = std::panic::AssertUnwindSafe(async {
      // meta
      let libraries = meta.libraries;
      let asset_index = instance.asset.get_index(&meta.asset_index).await?;
      let client = meta.downloads.client;

      let libraries_size = instance.library.get_download_size(&libraries).await;
      let assets_size = instance.asset.get_download_size(&asset_index).await;
      let client_size = instance.library.get_client_download_size(&client, &meta.id).await;
      let total = libraries_size.saturating_add(assets_size).saturating_add(client_size);

      if total == 0 {
        return Ok(Some(()));
      }

      download.set_total(total);
      download.set_downloading();
      download.start();

      // libs
      let lib_tasks = build_lib_tasks(&instance, &download, libraries);
      let lib_results = future::try_join_all(lib_tasks).await.map_err(|e| crate::Error::Download(e.to_string()))?;

      for (lib, result) in lib_results {
        match result {
          Ok(DownloadResult::Cancelled) => return Ok(None),
          Ok(_) => {}
          Err(e) => {
            return Err(crate::Error::Download(format!("Couldn't download library {}: {}", lib.name, e)));
          }
        }
      }

      // assets
      let asset_tasks = build_asset_tasks(&instance, &download, asset_index);
      let asset_results = future::try_join_all(asset_tasks).await.map_err(|e| crate::Error::Download(e.to_string()))?;

      for (asset, result) in asset_results {
        match result {
          Ok(DownloadResult::Cancelled) => return Ok(None),
          Ok(_) => {}
          Err(e) => {
            return Err(crate::Error::Download(format!("Couldn't download asset {}: {}", asset.hash, e)));
          }
        }
      }

      // client
      let client_task = build_client_task(&instance, &download, client, &meta.id);
      let (_client, result) = client_task.await.map_err(|e| crate::Error::Download(e.to_string()))?;
      match result {
        Ok(DownloadResult::Cancelled) => return Ok(None),
        Ok(DownloadResult::Success) => (),
        Ok(DownloadResult::Ignored) => (),
        Ok(DownloadResult::Impossible) => (),
        Err(e) => {
          return Err(crate::Error::Download(format!("Couldn't download game client for {}: {}", meta.id, e)));
        }
      };

      Ok(Some(()))
    })
    .catch_unwind()
    .await
    .unwrap_or_else(|panic| {
      let msg = if let Some(s) = panic.downcast_ref::<&str>() {
        (*s).to_string()
      } else if let Some(s) = panic.downcast_ref::<String>() {
        s.clone()
      } else {
        "unknown panic payload".to_string()
      };
      Err(crate::Error::Download(format!("download_game panicked: {msg}")))
    });

    match download_result {
      Ok(None) => download.aborted(),
      Ok(Some(())) => download.success(),
      Err(e) => {
        download.set_error_message(e.to_string());
        download.failed()
      }
    };
  });
}

fn build_lib_tasks(
  instance: &Instance,
  download: &DownloadTask,
  libraries: Vec<ProcessedLibrary>,
) -> Vec<DownloadJob<ProcessedLibrary>> {
  let mut tasks = Vec::new();

  for lib in libraries {
    if lib.must_skip {
      continue;
    }

    let instance = instance.clone();
    let download = download.clone();
    tasks.push(tokio::spawn(async move {
      let result = async {
        if download.is_aborted() {
          return Ok(DownloadResult::Cancelled);
        }

        let path = instance.library.get_lib_path(&lib);
        if path.exists() {
          let mut valid = true;
          if let Some(expected_sha1) = lib.sha1.as_deref() {
            valid = crate::fs::is_valid_sha1(&path, expected_sha1).await?;
          } else if let Some(expected_size) = lib.size {
            let metadata = tokio::fs::metadata(&path).await.map_err(|e| crate::fs::IoError::from_io(e, &path))?;
            valid = metadata.len() == expected_size;
          }

          if valid {
            return Ok(DownloadResult::Ignored);
          }

          crate::fs::delete_file(&path).await.ok();
        }

        let url = match &lib.url {
          Some(url) => url,
          None => {
            // ! Library impossible to download; probably belongs to
            // ! an external version or contains malformed metadata
            return Ok(DownloadResult::Impossible);
          }
        };

        let net_permit = instance.net_s.clone().acquire_owned().await.unwrap();
        download.set_message(&lib.path);
        let bytes = crate::net::get_bytes(url).await?;
        drop(net_permit);

        let io_permit = instance.io_s.clone().acquire_owned().await.unwrap();
        crate::fs::write(&path, &bytes).await?;
        drop(io_permit);

        download.add_progress_msg(bytes.len() as u64, &lib.path);

        Ok(DownloadResult::Success)
      }
      .await;

      (lib, result)
    }));
  }

  tasks
}

fn build_asset_tasks(instance: &Instance, download: &DownloadTask, index: AssetIndex) -> Vec<DownloadJob<Asset>> {
  let mut tasks = Vec::new();

  for asset in index.objects {
    let instance = instance.clone();
    let download = download.clone();
    tasks.push(tokio::spawn(async move {
      let result = async {
        if download.is_aborted() {
          return Ok(DownloadResult::Cancelled);
        }

        let (url, path) = instance.asset.get_asset_url_and_path(&asset)?;

        if path.exists() {
          let valid = crate::fs::is_valid_sha1(&path, &asset.hash).await?;
          if valid {
            return Ok(DownloadResult::Ignored);
          }

          crate::fs::delete_file(&path).await.ok();
        }

        let net_permit = instance.net_s.clone().acquire_owned().await.unwrap();
        download.set_message(&asset.path);
        let bytes = crate::net::get_bytes(&url).await?;
        drop(net_permit);

        let io_permit = instance.io_s.clone().acquire_owned().await.unwrap();
        crate::fs::write(&path, &bytes).await?;
        drop(io_permit);

        download.add_progress_msg(bytes.len() as u64, &asset.path);

        Ok(DownloadResult::Success)
      }
      .await;

      (asset, result)
    }));
  }

  tasks
}

fn build_client_task(instance: &Instance, download: &DownloadTask, client: File, v_id: &str) -> DownloadJob<File> {
  let v_id = v_id.to_string();
  let instance = instance.clone();
  let download = download.clone();
  tokio::spawn(async move {
    let result = async {
      if download.is_aborted() {
        return Ok(DownloadResult::Cancelled);
      }

      let path = instance.library.get_client_path(&v_id);

      if path.exists() {
        let valid = crate::fs::is_valid_sha1(&path, &client.sha1).await?;
        if valid {
          return Ok(DownloadResult::Ignored);
        }

        crate::fs::delete_file(&path).await.ok();
      }

      let message = format!("{}.jar", v_id);
      download.set_message(&message);

      let jar_res = crate::net::get(&client.url).await?;
      crate::util::stream_to_file::<crate::Error>(
        jar_res,
        &path,
        || download.is_aborted(),
        |size| download.add_progress_msg(size as u64, &message),
      )
      .await?;

      Ok(DownloadResult::Success)
    }
    .await;

    (client, result)
  })
}
