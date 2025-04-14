use super::{Arguments, Library, Logging, version::VersionType};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChildVersionMeta {
  pub id: String,
  pub inherits_from: String,
  #[serde(rename = "type")]
  pub type_: VersionType,
  pub main_class: Option<String>,
  pub arguments: Option<Arguments>,
  pub minecraft_arguments: Option<String>, // arguments (legacy and no jvm)
  pub libraries: Vec<Library>,
  pub release_time: String,
  pub time: String,
  pub logging: Option<Logging>,
}
