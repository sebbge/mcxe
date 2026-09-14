use super::{FullVersionMeta, PartialVersionMeta};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize, de::Deserializer};

//===> VERSION MANIFEST
fn manifest_versions_vec_to_indexmap<'de, D>(deserializer: D) -> Result<IndexMap<String, Version>, D::Error>
where
  D: Deserializer<'de>,
{
  let vec = Vec::<ManifestVersion>::deserialize(deserializer)?;
  Ok(vec.into_iter().map(|mv| (mv.id.clone(), Version::from(mv))).collect())
}

#[derive(Default, Deserialize, Debug)]
pub struct VersionManifest {
  pub latest: LatestVersions,
  #[serde(deserialize_with = "manifest_versions_vec_to_indexmap")]
  pub versions: IndexMap<String, Version>,
}

#[cfg_attr(feature = "ts-types", derive(ts_rs::TS), ts(export))]
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct LatestVersions {
  pub release: String,
  pub snapshot: String,
}
impl LatestVersions {
  pub fn is_latest(&self, version_id: &str) -> bool {
    self.release == version_id || self.snapshot == version_id
  }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ManifestVersion {
  pub id: String,
  #[serde(rename = "type")]
  pub type_: VersionType,
  pub url: String,
  // pub time: String,
  // pub release_time: String,
}

//===> VERSION METADATA
#[cfg_attr(feature = "ts-types", derive(ts_rs::TS), ts(export))]
#[derive(Serialize, Debug, Clone)]
pub struct Version {
  pub id: String,
  pub name: String,
  pub inherits_from: Option<String>,
  #[serde(rename = "type")]
  pub type_: VersionType,

  #[serde(skip_serializing)]
  #[cfg_attr(feature = "ts-types", ts(skip))]
  pub url: Option<String>,

  pub is_external: bool,
  pub status: VersionStatus,
}

#[cfg_attr(feature = "ts-types", derive(ts_rs::TS), ts(export))]
#[derive(Serialize, Default, Debug, Clone)]
pub enum VersionStatus {
  #[default]
  Unchecked,
  NeedsDownload,
  Ready,
}

#[cfg_attr(feature = "ts-types", derive(ts_rs::TS), ts(export))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
  OldAlpha,
  OldBeta,
  Release,
  Snapshot,
}

impl From<ManifestVersion> for Version {
  /// Convert vanilla versions to [`Version`]
  fn from(v: ManifestVersion) -> Self {
    Self {
      name: format!("Minecraft {}", v.id),
      inherits_from: None,
      id: v.id,
      type_: v.type_,
      url: Some(v.url),
      is_external: false,
      status: VersionStatus::Unchecked,
    }
  }
}

impl From<PartialVersionMeta> for Version {
  /// Convert **external**  versions with **partial** metadata to [`Version`]
  fn from(v: PartialVersionMeta) -> Self {
    let name =
      v.id.split('-').filter(|&part| part != v.inherits_from && part != "loader").collect::<Vec<&str>>().join(" ");

    Self {
      name: capitalize(&name),
      inherits_from: Some(v.inherits_from),
      id: v.id,
      type_: v.type_,
      url: None,
      is_external: true,
      status: VersionStatus::Unchecked,
    }
  }
}

impl From<FullVersionMeta> for Version {
  /// Convert **external**  versions with **full** metadata to [`Version`]
  /// - Vanilla versions must NOT be converted this way.
  fn from(v: FullVersionMeta) -> Self {
    Self {
      name: v.id.clone(),
      inherits_from: None,
      id: v.id,
      type_: v.type_,
      url: None,
      is_external: true,
      status: VersionStatus::Unchecked,
    }
  }
}

pub fn capitalize(s: &str) -> String {
  let mut caracteres = s.chars();
  match caracteres.next() {
    Some(c) => c.to_uppercase().collect::<String>() + caracteres.as_str(),
    None => String::new(),
  }
}
