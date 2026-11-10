use std::{path::PathBuf, sync::Arc};
use tokio::sync::Semaphore;

const DEFAULT_LAUNCHER_NAME: &str = "mcxe";
const DEFAULT_LAUNCHER_VERSION: &str = "1.0.0";
const DEFAULT_ROOT_DIR: &str = "./mcxe";
const DEFAULT_ASSETS_URL: &str = "https://resources.download.minecraft.net/";
const DEFAULT_MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest.json";

#[derive(bon::Builder)]
#[builder(on(_, into))]
pub struct LauncherConfig {
  #[builder(default = DEFAULT_LAUNCHER_NAME)]
  pub(super) name: String,
  #[builder(default = DEFAULT_LAUNCHER_VERSION)]
  pub(super) version: String,
  #[builder(default = DEFAULT_ROOT_DIR)]
  pub(super) root_dir: PathBuf,
  #[builder(default = DEFAULT_ASSETS_URL)]
  pub(super) assets_url: String,
  #[builder(default = DEFAULT_MANIFEST_URL)]
  pub(super) manifest_url: String,
  pub(super) net_s: Option<Arc<Semaphore>>,
  pub(super) io_s: Option<Arc<Semaphore>>,
}

impl Default for LauncherConfig {
  fn default() -> Self {
    Self::builder().build()
  }
}
