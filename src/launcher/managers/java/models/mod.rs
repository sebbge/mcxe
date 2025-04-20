use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct ZuluPackage {
  // arch: String,
  pub ext: String,
  // java_version: Vec<u32>,
  // latest: bool,
  // md5_hash: String,
  // name: String,
  // os: String,
  // sha256_hash: String,
  pub size: u64,
  // support_term: String,
  pub url: String,
}
