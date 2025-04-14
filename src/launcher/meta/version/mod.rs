mod version_manifest;
pub use version_manifest::*;

use super::models::{
  child::ChildVersionMeta,
  vanilla::FullVersionMeta,
  version::{Version, VersionMeta, VersionStatus},
};
use indexmap::IndexMap;
use std::path::{Path, PathBuf};

/// Version Manager
/// todo: docs
pub struct VersionManager {
  pub versions_dir: PathBuf,
  manifest: version_manifest::ManifestManager,
  versions: IndexMap<String, Version>,
}
impl VersionManager {
  pub fn new(versions_dir: PathBuf, manifest_config: ManifestConfig) -> Self {
    let manifest = ManifestManager::new(&versions_dir, manifest_config);

    Self {
      versions_dir,
      manifest,
      versions: IndexMap::new(),
    }
  }

  //

  pub async fn meta(&self, version_id: &str) -> crate::Result<VersionMeta> {
    let version = match self.get_one(version_id) {
      Some(v) => v,
      None => {
        return Err(crate::Error::Launcher(
          "The required metadata version does not exist or is not loaded.".into(),
        ));
      }
    };

    let meta_path = self.versions_dir.join(&version.id).join(format!("{version_id}.json"));

    if let Some(local_meta) = Self::try_read(&meta_path).await? {
      return Ok(local_meta);
    }

    let version_url = match &version.url {
      Some(url) => url,
      None => {
        return Err(crate::Error::Launcher(
          "The required metadata does not exist and there is no URL to retrieve it.".into(),
        ));
      }
    };

    let fresh_meta_string = crate::net::get_text(version_url).await?;
    let fresh_meta = match serde_json::from_str::<FullVersionMeta>(&fresh_meta_string) {
      Ok(full) => VersionMeta::Full(Box::new(full)),
      Err(_) => {
        let partial_meta = serde_json::from_str::<ChildVersionMeta>(&fresh_meta_string)?;
        VersionMeta::Partial(Box::new(partial_meta))
      }
    };

    crate::io::write_string(&meta_path, &fresh_meta_string).await?;

    Ok(fresh_meta)
  }

  async fn try_read(meta_path: &Path) -> crate::Result<Option<VersionMeta>> {
    if !meta_path.exists() {
      return Ok(None);
    }

    let meta_bytes = crate::io::read_bytes(&meta_path).await?;

    match serde_json::from_slice::<FullVersionMeta>(&meta_bytes) {
      Ok(full) => {
        let meta = VersionMeta::Full(Box::new(full));
        Ok(Some(meta))
      }
      Err(_) => {
        let partial = serde_json::from_slice::<ChildVersionMeta>(&meta_bytes)?;
        let meta = VersionMeta::Partial(Box::new(partial));
        Ok(Some(meta))
      }
    }
  }

  pub fn get_one(&self, version_id: &str) -> Option<&Version> {
    self.versions.get(version_id)
  }

  pub async fn load_versions(&mut self) -> crate::Result<()> {
    let mut manifest = self.manifest.get().await?;
    let mut external_versions = IndexMap::new();

    let version_folders = crate::io::get_subfolders(&self.versions_dir).await?;
    for version_id in version_folders {
      let version_path = self.versions_dir.join(&version_id).join(format!("{version_id}.json"));

      if let Some(version) = manifest.versions.get_mut(&version_id) {
        if !version_path.exists() {
          println!("{:#?}", version_path);
          version.status = VersionStatus::NeedsDownload;
          continue;
        }

        //TODO: Uncomment when ready
        // version.meta = match manifest.latest.is_latest(&version_id) {
        //   true => crate::io::read_and_parse_file::<FullVersionMeta>(&version_path)
        //     .await
        //     .ok()
        //     .map(|meta| meta.p),
        //   false => None,
        // };
      } else {
        // Expects full or partial/child version meta
        match VersionManager::try_read(&version_path).await {
          Ok(None) => continue, //version_path does not exists -> ignore version
          Ok(Some(meta)) => {
            let version = Version::from(meta);
            external_versions.insert(version.id.clone(), version);
          }
          Err(e) => {
            tracing::error!(
              target = "VersionManager",
              "Couldn't load an alleged external version metadata {version_id}: {e}"
            );
            continue;
          }
        }
      }
    }

    self.versions = external_versions;
    self.versions.extend(manifest.versions);

    Ok(())
  }
}
