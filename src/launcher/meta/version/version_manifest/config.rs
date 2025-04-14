use std::{collections::HashSet, path::PathBuf, time::Duration};

pub struct ManifestConfig {
  pub url: String,
  pub path: Option<PathBuf>,
  pub refresh_interval: Option<Duration>,
  pub unsupported_versions: Option<HashSet<String>>,
}
impl ManifestConfig {
  pub fn builder() -> ManifestConfigBuilder {
    ManifestConfigBuilder::default()
  }
}

#[derive(Default)]
pub struct ManifestConfigBuilder {
  url: Option<String>,
  path: Option<PathBuf>,
  refresh_interval: Option<Duration>,
  unsupported_versions: Option<HashSet<String>>,
}

impl ManifestConfigBuilder {
  pub fn url(mut self, url: impl Into<String>) -> Self {
    self.url = Some(url.into());
    self
  }

  pub fn path(mut self, path: impl Into<PathBuf>) -> Self {
    self.path = Some(path.into());
    self
  }

  pub fn refresh_interval(mut self, interval: Duration) -> Self {
    self.refresh_interval = Some(interval);
    self
  }

  pub fn unsupported_versions(mut self, versions: HashSet<String>) -> Self {
    self.unsupported_versions = Some(versions);
    self
  }

  pub fn build(self) -> crate::Result<ManifestConfig> {
    Ok(ManifestConfig {
      url: self.url.ok_or(crate::Error::Build(
        "Minecraft version manifest URL is required.".into(),
      ))?,
      path: self.path,
      refresh_interval: self.refresh_interval,
      unsupported_versions: self.unsupported_versions,
    })
  }
}
