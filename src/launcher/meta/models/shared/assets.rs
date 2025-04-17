use serde::{Deserialize, de::Deserializer};
use std::collections::HashMap;

fn asset_map_to_vec<'de, D>(deserializer: D) -> Result<Vec<Asset>, D::Error>
where
  D: Deserializer<'de>,
{
  let map: HashMap<String, RawAsset> = HashMap::deserialize(deserializer)?;
  Ok(
    map
      .into_iter()
      .map(|(path, raw)| Asset {
        path,
        hash: raw.hash,
        size: raw.size,
      })
      .collect(),
  )
}

#[derive(Deserialize)]
pub struct RawAsset {
  pub hash: String,
  pub size: u64,
}

#[derive(Deserialize, Debug)]
pub struct AssetIndex {
  pub map_to_resources: Option<bool>,
  #[serde(deserialize_with = "asset_map_to_vec")]
  pub objects: Vec<Asset>,
}

#[derive(Deserialize, Debug)]
pub struct Asset {
  pub path: String,
  pub hash: String,
  pub size: u64,
}
