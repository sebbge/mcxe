mod arguments;
mod assets;
mod library;
mod rule;
pub use arguments::*;
pub use assets::*;
pub use library::*;
pub use rule::*;

use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Action {
  Allow,
  Disallow,
}

#[derive(Deserialize, Debug, Clone)]
pub struct File {
  pub sha1: String,
  pub size: i64,
  pub url: String,
  pub path: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndexFile {
  pub id: String,
  pub sha1: String,
  pub size: usize,
  pub total_size: usize,
  pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct Logging {
  pub client: LoggingClient,
}
#[derive(Deserialize, Debug)]
pub struct LoggingClient {
  pub argument: String, // logging argument, always "-Dlog4j.configurationFile=${path}"
  pub file: File,
  #[serde(rename = "type")]
  pub type_: String, // always "log4j2-xml"
}
