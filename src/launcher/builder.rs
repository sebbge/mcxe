use crate::{
  Launcher,
  managers::{AssetManager, ClientManager, JavaManager, LibraryManager, ManifestConfig, VersionManager},
};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::RwLock;
use url::Url;

#[derive(Default)]
pub struct LauncherBuilder {
  launcher_name: Option<String>,
  launcher_version: Option<String>,
  root_dir: Option<PathBuf>,
  games_dir: Option<PathBuf>,
  java_dir: Option<PathBuf>,
  versions_dir: Option<PathBuf>,
  libraries_dir: Option<PathBuf>,
  assets_dir: Option<PathBuf>,
  natives_dir: Option<PathBuf>,
  manifest_config: Option<ManifestConfig>,
  assets_url: Option<String>,
}

impl LauncherBuilder {
  pub fn name(mut self, name: impl Into<String>) -> Self {
    self.launcher_name = Some(name.into());
    self
  }

  pub fn version(mut self, version: impl Into<String>) -> Self {
    self.launcher_version = Some(version.into());
    self
  }

  pub fn root_dir(mut self, path: impl Into<PathBuf>) -> Self {
    self.root_dir = Some(path.into());
    self
  }

  pub fn games_dir(mut self, path: impl Into<PathBuf>) -> Self {
    self.games_dir = Some(path.into());
    self
  }

  pub fn java_dir(mut self, path: impl Into<PathBuf>) -> Self {
    self.java_dir = Some(path.into());
    self
  }

  pub fn versions_dir(mut self, path: impl Into<PathBuf>) -> Self {
    self.versions_dir = Some(path.into());
    self
  }

  pub fn libraries_dir(mut self, path: impl Into<PathBuf>) -> Self {
    self.libraries_dir = Some(path.into());
    self
  }

  pub fn assets_dir(mut self, path: impl Into<PathBuf>) -> Self {
    self.assets_dir = Some(path.into());
    self
  }

  pub fn natives_dir(mut self, path: impl Into<PathBuf>) -> Self {
    self.natives_dir = Some(path.into());
    self
  }

  pub fn manifest_config(mut self, manifest_config: ManifestConfig) -> Self {
    self.manifest_config = Some(manifest_config);
    self
  }

  pub fn assets_url(mut self, assets_url: impl Into<String>) -> Self {
    self.assets_url = Some(assets_url.into());
    self
  }

  pub fn build(self) -> crate::Result<Launcher> {
    let launcher_name = self
      .launcher_name
      .ok_or(crate::Error::Build("Launcher name is required".into()))?;

    let launcher_version = self
      .launcher_version
      .ok_or(crate::Error::Build("Launcher version is required".into()))?;

    let root_dir = self
      .root_dir
      .ok_or(crate::Error::Build("Root directory is required".into()))?;
    let games_dir = self.games_dir.unwrap_or_else(|| root_dir.join("instances"));
    let libraries_dir = self.libraries_dir.unwrap_or_else(|| root_dir.join("libraries"));
    let assets_dir = self.assets_dir.unwrap_or_else(|| root_dir.join("assets"));
    let natives_dir = self.natives_dir.unwrap_or_else(|| root_dir.join("natives"));

    //= Library manager
    let library = Arc::new(LibraryManager::new(libraries_dir, natives_dir));

    //= Version manager
    let config = self
      .manifest_config
      .ok_or(crate::Error::Build("Version manifest config is required".into()))?;
    let versions_dir = self.versions_dir.unwrap_or_else(|| root_dir.join("versions"));
    let version = Arc::new(RwLock::new(VersionManager::new(versions_dir.clone(), config)));

    //= Client manager
    let client = Arc::new(ClientManager::new(versions_dir));

    //= Asset manager
    let assets_url = self
      .assets_url
      .ok_or(crate::Error::Build("Asset URL is required".into()))?;
    let assets_url = Url::parse(&assets_url).map_err(|e| crate::Error::Build(format!("Invalid asset URL: {}", e)))?;
    let asset = Arc::new(AssetManager::new(assets_dir, assets_url));

    //= Java manager
    let java_dir = self.java_dir.unwrap_or_else(|| root_dir.join("java"));
    let java = Arc::new(RwLock::new(JavaManager::new(java_dir)));

    Ok(Launcher {
      launcher_name,
      launcher_version,
      root_dir,
      games_dir,
      //
      version,
      library,
      client,
      asset,
      java,
    })
  }
}
