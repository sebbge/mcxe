use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct AssetIndex {
  pub map_to_resources: Option<bool>, //?? what is this
  pub objects: Assets,
}

pub type Assets = HashMap<String, Asset>;

#[derive(Deserialize, Debug)]
pub struct Asset {
  pub hash: String,
  pub size: usize,
}
