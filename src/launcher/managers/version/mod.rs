mod manifest;
pub use manifest::*;
use serde::de::DeserializeOwned;

use crate::LatestVersions;
use crate::meta::{FullVersionMeta, PartialVersionMeta, ProcessedVersionMeta, Version, VersionManifest, VersionStatus};
use indexmap::IndexMap;
use std::path::{Path, PathBuf};
use url::Url;

pub struct VersionManager {
  pub versions_dir: PathBuf,
  manifest: ManifestManager,
  versions: IndexMap<String, Version>,
  latest_versions: Option<LatestVersions>,
}
impl VersionManager {
  pub fn new(versions_dir: PathBuf, manifest_url: Url) -> Self {
    let manifest = ManifestManager::new(&versions_dir, manifest_url);

    Self {
      versions_dir,
      manifest,
      versions: IndexMap::new(),
      latest_versions: None,
    }
  }

  pub async fn init(&mut self) -> crate::Result<()> {
    let mut manifest = self.manifest.get().await.unwrap_or_else(|e| {
      tracing::error!("Couldn't get minecraft version manifest: {}", e);
      VersionManifest::default()
    });
    let mut external_versions = IndexMap::new();

    let version_folders = crate::fs::get_subfolders(&self.versions_dir).await?;
    for version_id in version_folders {
      let version_path = self.get_version_path(&version_id);

      if let Some(version) = manifest.versions.get_mut(&version_id) {
        if !version_path.exists() {
          version.status = VersionStatus::NeedsDownload;
          continue;
        } else {
          // cache
          // version.meta = match manifest.latest.is_latest(&version_id) {
          //   true => crate::fs::read_json::<FullVersionMeta>(&version_path)
          //     .await
          //     .ok()
          //     .map(|meta| meta.p),
          //   false => None,
          // };
        }
      } else {
        //- expects full or partial/child version meta
        let bytes = match crate::fs::read(&version_path).await {
          Ok(b) => b,
          Err(e) => {
            tracing::error!("Couldn't load version {}: {}", &version_id, e);
            continue;
          }
        };

        let version = match serde_json::from_slice::<FullVersionMeta>(&bytes) {
          Ok(meta) => Version::from(meta),
          Err(_) => {
            let full = match serde_json::from_slice::<PartialVersionMeta>(&bytes) {
              Ok(meta) => meta,
              Err(e) => {
                tracing::error!("Version {} contains malformed metadata: {}", &version_id, e);
                continue;
              }
            };

            Version::from(full)
          }
        };

        external_versions.insert(version.id.clone(), version);
      }
    }

    self.versions = external_versions;
    self.versions.extend(manifest.versions);
    self.latest_versions = Some(manifest.latest);

    Ok(())
  }

  pub fn versions(&self) -> Vec<Version> {
    self.versions.values().cloned().collect()
  }

  pub async fn latest_versions(&self) -> LatestVersions {
    if let Some(latest) = &self.latest_versions {
      return latest.clone();
    }
    self.manifest.get().await.map(|m| m.latest).unwrap_or_else(|e| {
      tracing::error!("Couldn't get latest minecraft versions: {}", e);
      LatestVersions::default()
    })
  }
  //

  pub fn get_version_path(&self, version_id: &str) -> PathBuf {
    self.versions_dir.join(version_id).join(format!("{}.json", version_id))
  }

  pub fn resolve(&self, version_id: &str) -> crate::Result<(Option<&Version>, &Version)> {
    let version = self
      .versions
      .get(version_id)
      .ok_or_else(|| crate::Error::Metadata("The required version does not exist or was not loaded.".into()))?;

    if let Some(parent_id) = &version.inherits_from {
      let parent_version = self.versions.get(parent_id).ok_or_else(|| {
        crate::Error::Metadata("The required version parent does not exist or was not loaded.".into())
      })?;
      Ok((Some(parent_version), version))
    } else {
      Ok((None, version))
    }
  }

  async fn fetch_and_write<T: DeserializeOwned>(&self, url: &Option<String>, path: &Path) -> crate::Result<T> {
    let url = url.as_ref().ok_or_else(|| {
      crate::Error::Metadata("The required metadata does not exist and there is no URL to retrieve it.".into())
    })?;

    let fresh_meta_bytes = crate::net::get_bytes(url).await?;
    let fresh_meta = serde_json::from_slice::<T>(&fresh_meta_bytes)?;
    crate::fs::write(&path, &fresh_meta_bytes).await?;

    Ok(fresh_meta)
  }

  async fn read_or_fetch<T: DeserializeOwned>(&self, version: &Version) -> crate::Result<T> {
    let path = self.get_version_path(&version.id);

    if !path.exists() {
      let fresh_meta = self.fetch_and_write::<T>(&version.url, &path).await?;
      return Ok(fresh_meta);
    }

    let bytes = crate::fs::read(&path).await?;
    let meta = serde_json::from_slice::<T>(&bytes)?;
    Ok(meta)
  }

  //

  pub async fn meta(&self, version_id: &str) -> crate::Result<ProcessedVersionMeta> {
    let (parent, version) = self.resolve(version_id)?;

    if let Some(parent) = parent {
      let partial = self.read_or_fetch::<PartialVersionMeta>(version).await?;
      let full = self.read_or_fetch::<FullVersionMeta>(parent).await?;
      let meta = full.merge_and_process(partial);
      //cache version.meta = Some(meta.clone());
      Ok(meta)
    } else {
      let full = self.read_or_fetch::<FullVersionMeta>(version).await?;
      let meta = full.process();
      //cache version.meta = Some(meta.clone());
      Ok(meta)
    }
  }
}
