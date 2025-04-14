use super::{child::ChildVersionMeta, processed::ProcessedVersionMeta, vanilla::FullVersionMeta};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize, de::Deserializer};
use shared::util;

//====VERSION MANIFEST====//

/// Custom deserializer that converts a JSON array of [`ManifestVersion`] into an [`IndexMap<String, Version>`].
///
/// Each [`ManifestVersion`] is converted into a [`Version`] using the [`From<ManifestVersion>`] implementation.
fn manifest_versions_vec_to_indexmap<'de, D>(deserializer: D) -> Result<IndexMap<String, Version>, D::Error>
where
  D: Deserializer<'de>,
{
  let vec = Vec::<ManifestVersion>::deserialize(deserializer)?;
  Ok(vec.into_iter().map(|mv| (mv.id.clone(), Version::from(mv))).collect())
}

#[derive(Deserialize, Debug)]
pub struct VersionManifest {
  pub latest: LatestVersions,
  #[serde(deserialize_with = "manifest_versions_vec_to_indexmap")]
  pub versions: IndexMap<String, Version>,
}

#[derive(Default, Deserialize, Debug)]
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

//====VERSION METADATA====//

#[derive(Debug)]
pub enum VersionMeta {
  /// Full version metadata
  /// Refers to a vanilla version meta with no dependencies.
  Full(Box<FullVersionMeta>),

  /// Partial/Child version metadata
  /// Refers to an external version meta that depends on a full vanilla meta.
  /// Should be merged/processed with a full meta to be used.
  Partial(Box<ChildVersionMeta>),
  // Processed version metadata (ready for launch/download)
  // Obtained by processing a full metadata or merging partial meta with a full one.
  // All arguments and libraries are processed according to their rules.
  // Processed(Box<ProcessedVersionMeta>),
}

#[derive(Serialize, Default, Debug)]
pub enum VersionStatus {
  #[default]
  Unchecked,
  NeedsDownload,
  Ready,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
  OldAlpha,
  OldBeta,
  Release,
  Snapshot,
}

#[derive(Serialize, Debug)]
pub struct Version {
  pub id: String,
  pub name: String,
  #[serde(rename = "type")]
  pub type_: VersionType,

  #[serde(skip_serializing)]
  pub url: Option<String>,

  pub is_external: bool,
  pub status: VersionStatus,

  #[serde(skip)]
  /// Version meta cache
  pub meta: Option<ProcessedVersionMeta>,
}

impl From<ManifestVersion> for Version {
  /// Convert vanilla versions to [`Version`]
  fn from(v: ManifestVersion) -> Self {
    Self {
      name: format!("Minecraft {}", &v.id),
      id: v.id,
      type_: v.type_,
      url: Some(v.url),
      is_external: false,
      status: VersionStatus::Unchecked,
      meta: None,
    }
  }
}

impl From<ChildVersionMeta> for Version {
  /// Convert **external**  versions with **partial** metadata to [`Version`]
  fn from(v: ChildVersionMeta) -> Self {
    let name = v
      .id
      .split('-')
      .filter(|&part| part != v.inherits_from && part != "loader")
      .collect::<Vec<&str>>()
      .join(" ");

    Self {
      name: util::format::capitalize(&name),
      id: v.id,
      type_: v.type_,
      url: None,
      is_external: true,
      status: VersionStatus::Unchecked,
      meta: None,
    }
  }
}

impl From<FullVersionMeta> for Version {
  /// Convert **external**  versions with **full** metadata to [`Version`]
  /// - Vanilla versions must NOT be converted this way.
  fn from(v: FullVersionMeta) -> Self {
    Self {
      name: v.id.clone(),
      id: v.id,
      type_: v.type_,
      url: None,
      is_external: true,
      status: VersionStatus::Unchecked,
      meta: None,
    }
  }
}

impl From<VersionMeta> for Version {
  fn from(v: VersionMeta) -> Self {
    match v {
      VersionMeta::Full(v) => Version::from(*v),
      VersionMeta::Partial(v) => Version::from(*v),
    }
  }
}
