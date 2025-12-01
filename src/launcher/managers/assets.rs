use crate::meta::{Asset, AssetIndex, AssetIndexFile};
use std::path::{Path, PathBuf};
use url::Url;

pub struct AssetManager {
  pub assets_dir: PathBuf,
  pub assets_url: Url,
}
impl AssetManager {
  pub fn new(assets_dir: PathBuf, assets_url: url::Url) -> Self {
    Self { assets_dir, assets_url }
  }

  pub async fn get_index(&self, index_meta: &AssetIndexFile) -> crate::Result<AssetIndex> {
    let index_path = self.assets_dir.join("indexes").join(format!("{}.json", &index_meta.id));

    if let Some(local_index) = Self::try_read(&index_path).await? {
      return Ok(local_index);
    }

    let fresh_index_bytes = crate::net::get_bytes(&index_meta.url).await?;
    let fresh_index = serde_json::from_slice::<AssetIndex>(&fresh_index_bytes)?;

    crate::fs::write(&index_path, &fresh_index_bytes).await?;
    Ok(fresh_index)
  }

  async fn try_read(index_path: &Path) -> crate::Result<Option<AssetIndex>> {
    if !index_path.exists() {
      return Ok(None);
    }

    let index_bytes = crate::fs::read(&index_path).await?;
    let index = serde_json::from_slice::<AssetIndex>(&index_bytes)?;
    Ok(Some(index))
  }

  pub async fn get_download_size(&self, index: &AssetIndex) -> u64 {
    let mut total = 0;

    for asset in &index.objects {
      let prefix = &asset.hash[0..2];

      let asset_path = self.assets_dir.join("objects").join(prefix).join(&asset.hash);
      if !asset_path.exists() {
        total += asset.size;
      }
    }

    total
  }

  pub fn get_asset_url_and_path(&self, asset: &Asset) -> crate::Result<(String, PathBuf)> {
    if asset.hash.len() < 2 {
      return Err(crate::Error::Metadata(format!("Invalid asset hash: {}", &asset.hash)));
    }

    let prefix = &asset.hash[0..2];

    let url = self
      .assets_url
      .join(&format!("{}/", &prefix))
      .and_then(|url| url.join(&asset.hash))
      .map_err(|_| crate::Error::Metadata(format!("Invalid asset hash: {}", &asset.hash)))?;
    let path = self.assets_dir.join("objects").join(prefix).join(&asset.hash);
    Ok((url.to_string(), path))
  }
}
