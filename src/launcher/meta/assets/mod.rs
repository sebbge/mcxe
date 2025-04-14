use super::models::{AssetIndex, AssetIndexFile};
use std::path::{Path, PathBuf};

pub struct AssetManager {
  pub assets_dir: PathBuf,
}
impl AssetManager {
  pub fn new(assets_dir: PathBuf) -> Self {
    Self { assets_dir }
  }

  //

  pub async fn get_index(&self, index_meta: &AssetIndexFile) -> crate::Result<AssetIndex> {
    let index_path = self.assets_dir.join(format!("indexes/{}.json", &index_meta.id));

    if let Some(local_index) = Self::try_read(&index_path).await? {
      return Ok(local_index);
    }

    let fresh_index_string = crate::net::get_text(&index_meta.url).await?;
    let fresh_index = serde_json::from_str::<AssetIndex>(&fresh_index_string)?;

    crate::io::write_string(&index_path, &fresh_index_string).await?;

    Ok(fresh_index)
  }

  async fn try_read(index_path: &Path) -> crate::Result<Option<AssetIndex>> {
    if !index_path.exists() {
      return Ok(None);
    }

    let index_bytes = crate::io::read_bytes(&index_path).await?;

    let index = serde_json::from_slice::<AssetIndex>(&index_bytes)?;

    Ok(Some(index))
  }
}
