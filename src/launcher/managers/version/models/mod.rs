use crate::meta::VersionManifest;
use std::{
  path::{Path, PathBuf},
  time::Duration,
};
use url::Url;

/// Manifest Manager
/// todo: docs
pub struct ManifestManager {
  pub url: Url,
  pub path: PathBuf,
  refresh: Duration,
}
impl ManifestManager {
  pub fn new(versions_dir: &Path, url: Url, path: Option<PathBuf>, refresh: Option<Duration>) -> Self {
    let path = path.unwrap_or_else(|| versions_dir.join("version_manifest.json"));
    let refresh = refresh.unwrap_or_else(|| Duration::from_secs(60 * 60 * 24));

    Self { url, path, refresh }
  }
  //

  pub async fn get(&self) -> crate::Result<VersionManifest> {
    if let Some(local_manifest) = self.try_read().await? {
      return Ok(local_manifest);
    }

    let fresh_manifest_bytes = crate::net::get_bytes(self.url.as_ref()).await?;
    let fresh_manifest = serde_json::from_slice::<VersionManifest>(&fresh_manifest_bytes)?;

    crate::fs::write(&self.path, &fresh_manifest_bytes).await?;

    Ok(fresh_manifest)
  }

  async fn try_read(&self) -> crate::Result<Option<VersionManifest>> {
    if !self.path.exists() || crate::fs::is_old(&self.path, &self.refresh).await? {
      return Ok(None);
    }

    let manifest_bytes = crate::fs::read(&self.path).await?;
    let manifest = serde_json::from_slice::<VersionManifest>(&manifest_bytes)?;

    Ok(Some(manifest))
  }
}
