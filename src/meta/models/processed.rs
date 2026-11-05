use super::{AssetIndexFile, ClientDownloads, JavaVersion, Logging, VersionType};

#[derive(Debug, Clone)]
pub struct ProcessedVersionMeta {
  pub id: String,
  pub inherits_from: Option<String>,
  pub type_: VersionType,
  pub asset_index: AssetIndexFile,
  pub assets: String,
  pub java_version: Option<JavaVersion>,
  pub compliance_level: Option<u16>,
  pub downloads: ClientDownloads,
  pub main_class: String,
  pub arguments: ProcessedArguments,
  pub libraries: Vec<ProcessedLibrary>,
  pub minimum_launcher_version: u16,
  pub release_time: String,
  pub time: String,
  pub logging: Option<Logging>,
}

impl ProcessedVersionMeta {
  pub fn client_id(&self) -> &str {
    self.inherits_from.as_deref().unwrap_or(&self.id)
  }
}

#[derive(Default, Debug, Clone)]
pub struct ProcessedArguments {
  pub game: Vec<String>,
  pub jvm: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProcessedLibrary {
  pub name: String,
  pub path: String,
  pub url: Option<String>,
  pub sha1: Option<String>,
  pub size: Option<u64>,
  pub must_skip: bool,
  pub is_native: bool,
}
