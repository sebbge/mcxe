mod config;
pub use config::ManifestConfig;

use crate::launcher::meta::models::version::VersionManifest;
use std::{path::PathBuf, time::Duration};

/// Manifest Manager
/// todo: docs
pub struct ManifestManager {
  pub manifest_url: String,
  pub manifest_path: PathBuf,
  refresh_interval: Duration,
}
impl ManifestManager {
  pub fn new(versions_dir: &std::path::Path, config: ManifestConfig) -> Self {
    let manifest_path = config.path.unwrap_or(versions_dir.join("version_manifest.json"));
    let refresh_interval = config.refresh_interval.unwrap_or(Duration::from_secs(60 * 60 * 24));

    Self {
      manifest_url: config.url,
      manifest_path,
      refresh_interval,
    }
  }

  //

  pub async fn get(&self) -> crate::Result<VersionManifest> {
    if let Some(local_manifest) = self.try_read().await? {
      return Ok(local_manifest);
    }

    let fresh_manifest_string = crate::net::get_text(&self.manifest_url).await?;
    let fresh_manifest = serde_json::from_str::<VersionManifest>(&fresh_manifest_string)?;

    crate::io::write_string(&self.manifest_path, &fresh_manifest_string).await?;

    Ok(fresh_manifest)
  }

  async fn try_read(&self) -> crate::Result<Option<VersionManifest>> {
    if !self.manifest_path.exists() || crate::io::is_old(&self.manifest_path, &self.refresh_interval).await? {
      return Ok(None);
    }

    let manifest_bytes = crate::io::read_bytes(&self.manifest_path).await?;

    let manifest = serde_json::from_slice::<VersionManifest>(&manifest_bytes)?;

    Ok(Some(manifest))
  }
}
