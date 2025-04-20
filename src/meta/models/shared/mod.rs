mod arguments;
mod assets;
mod library;
mod rule;
pub use arguments::*;
pub use assets::*;
pub use library::*;
pub use rule::*;

use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Action {
  Allow,
  Disallow,
}
impl Action {
  pub fn into_bool(self) -> bool {
    match self {
      Action::Allow => true,
      Action::Disallow => false,
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct File {
  pub sha1: String,
  pub size: u64,
  pub url: String,
  pub path: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndexFile {
  pub id: String,
  pub sha1: String,
  pub size: u64,
  pub total_size: u64,
  pub url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Logging {
  pub client: LoggingClient,
}
#[derive(Deserialize, Debug, Clone)]
pub struct LoggingClient {
  pub argument: String, // logging argument, always "-Dlog4j.configurationFile=${path}"
  pub file: File,
  #[serde(rename = "type")]
  pub type_: String, // always "log4j2-xml"
}
