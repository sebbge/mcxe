use crate::meta::File;
use std::path::PathBuf;

pub struct ClientManager {
  pub client_dir: PathBuf,
}
impl ClientManager {
  pub fn new(client_dir: PathBuf) -> Self {
    Self { client_dir }
  }

  //
  pub async fn get_download_size(&self, client: &File, version_id: &str) -> u64 {
    let client_path = self.get_path(version_id);
    if !client_path.exists() { client.size } else { 0 }
  }

  pub fn get_path(&self, version_id: &str) -> PathBuf {
    self.client_dir.join(version_id).join(format!("{}.jar", version_id))
  }
}
