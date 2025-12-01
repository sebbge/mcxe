use super::{ProgressTask};
use crate::{Instance, meta::ProcessedVersionMeta};

use serde::Serialize;

#[derive(Default, Serialize, Debug)]
pub struct ChecksumReport {
  missing_libraries: Vec<String>,
  corrupted_libraries: Vec<String>,
  skipped_libraries: Vec<String>,
  missing_assets: Option<Vec<String>>,
  corrupted_assets: Option<Vec<String>>,
  errors: Vec<String>,
  download_required: bool,
}

pub(crate) fn checksum(
  instance: Instance,
  progress: ProgressTask,
  meta: ProcessedVersionMeta,
  check_assets: bool,
) {
  tokio::spawn(async move {
    let checksum_result: crate::Result<Option<ChecksumReport>> = async {
      let mut rep = ChecksumReport::default();
      progress.start().await;

      //--- Client
      let client_name = &format!("Client {}.jar", &meta.id);
      progress.set_message(client_name).await;
      let client_path = instance.library.get_client_path(&meta.id);
      let client_hash = Some(meta.downloads.client.sha1.as_ref());
      match check(&client_path, client_hash, client_name).await {
        Ok(()) => {}
        Err(CheckError::Missing(name)) => {
          rep.missing_libraries.push(name);
          rep.download_required = true;
        }
        Err(CheckError::Corrupt(name)) => {
          rep.corrupted_libraries.push(name);
          rep.download_required = true;
        }
      }

      //--- Libraries
      for lib in &meta.libraries {
        if progress.was_aborted() {
          return Ok(None);
        }

        if lib.must_skip {
          rep.skipped_libraries.push(lib.name.clone());
          continue;
        }

        progress.set_message(&lib.name).await;
        let lib_path = instance.library.get_lib_path(lib);
        match check(&lib_path, lib.sha1.as_deref(), &lib.name).await {
          Ok(()) => {}
          Err(CheckError::Missing(name)) => {
            rep.missing_libraries.push(name);
            rep.download_required = true;
          }
          Err(CheckError::Corrupt(name)) => {
            rep.corrupted_libraries.push(name);
            rep.download_required = true;
          }
        }
      }

      //--- Assets
      if check_assets {
        let mut missing_assets = Vec::new();
        let mut corrupted_assets = Vec::new();

        let asset_index = instance.asset.get_index(&meta.asset_index).await?;

        for asset in asset_index.objects {
          if progress.was_aborted() {
            return Ok(None);
          }

          progress.set_message(&asset.path).await;
          let asset_path = match instance.asset.get_asset_url_and_path(&asset) {
            Ok((_, path)) => path,
            Err(e) => {
              let msg = format!("Invalid asset {}: {}", asset.path, e);
              tracing::warn!(msg);
              corrupted_assets.push(asset.path.clone());
              rep.download_required = true;
              continue;
            }
          };
          match check(&asset_path, Some(&asset.hash), &asset.path).await {
            Ok(()) => {}
            Err(CheckError::Missing(name)) => {
              missing_assets.push(name);
              rep.download_required = true;
            }
            Err(CheckError::Corrupt(name)) => {
              corrupted_assets.push(name);
              rep.download_required = true;
            }
          }
        }

        rep.missing_assets = Some(missing_assets);
        rep.corrupted_assets = Some(corrupted_assets);
      }

      Ok(Some(rep))
    }
    .await;

    match checksum_result {
      Ok(None) => progress.aborted().await,
      Ok(Some(report)) => {
        let out = serde_json::to_string(&report).ok();
        progress.set_out(out).await;
        progress.success().await;
      }
      // TODO: error message
      Err(_e) => progress.failed().await,
    };
  });
}

enum CheckError {
  Missing(String),
  Corrupt(String),
}
async fn check(path: &std::path::Path, sha1_opt: Option<&str>, label: &str) -> Result<(), CheckError> {
  if !path.exists() {
    return Err(CheckError::Missing(label.to_string()));
  }

  if let Some(sha1) = sha1_opt {
    match crate::fs::is_valid_sha1(path, sha1).await {
      Ok(true) => return Ok(()),
      Ok(false) => return Err(CheckError::Corrupt(label.to_string())),
      Err(e) => {
        let msg = format!("Couldn't verify integrity for {}: {}", label, e);
        tracing::error!(msg);
        return Err(CheckError::Corrupt(label.to_string()));
      }
    };
  };

  Ok(())
}
