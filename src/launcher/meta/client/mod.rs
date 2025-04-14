use super::models::File;
use std::path::PathBuf;

pub struct ClientManager {
  pub client_dir: PathBuf,
}
impl ClientManager {
  pub fn new(client_dir: PathBuf) -> Self {
    Self { client_dir }
  }

  //

  pub async fn download(&self, client_meta: &File, version_id: &str) -> crate::Result<usize> {
    let client_path = self.client_dir.join(version_id).join(format!("{version_id}.json"));

    if client_path.exists() {
      return Ok(0);
    }

    let client_bytes = crate::net::get_bytes(&client_meta.url).await?;
    crate::io::write_bytes(&client_path, &client_bytes).await?;

    Ok(client_bytes.len())
  }

  //todo
  async fn check(&self, client_meta: &File, version_id: &str) -> crate::Result<()> {
    Ok(())
    // let client_path = self.client_dir.join(version_id).join(format!("{version_id}.json"));

    // if client_path.exists() { Ok(()) } else { Ok(()) }
  }
}
