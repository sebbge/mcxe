use super::{Arguments, AssetIndexFile, File, Library, Logging, VersionType};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FullVersionMeta {
  pub id: String,
  #[serde(rename = "type")]
  pub type_: VersionType,
  pub asset_index: AssetIndexFile, // asset index download
  pub assets: String,              // asset index id
  pub java_version: Option<JavaVersion>,
  pub compliance_level: Option<u16>, // player safety features
  pub downloads: ClientDownloads,
  pub main_class: String,
  pub arguments: Option<Arguments>,
  pub minecraft_arguments: Option<String>, // legacy arguments, no jvm
  pub libraries: Vec<Library>,
  pub minimum_launcher_version: u16,
  pub release_time: String,
  pub time: String,
  pub logging: Option<Logging>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
  // Can be:
  // "java-runtime-alpha"
  // "java-runtime-beta"
  // "java-runtime-delta"
  // "java-runtime-gamma"
  // "java-runtime-gamma-snapshot"
  // "jre-legacy"
  pub component: String,

  // java version
  pub major_version: u16,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ClientDownloads {
  pub client: File,
  pub server: Option<File>,
  pub client_mappings: Option<File>,
  pub server_mappings: Option<File>,
  pub windows_server: Option<File>,
}
