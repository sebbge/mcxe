use std::{path::PathBuf, sync::Arc};
use tokio::sync::Semaphore;

const DEFAULT_LAUNCHER_NAME: &str = "Comet";
const DEFAULT_LAUNCHER_VERSION: &str = "1.0.0";
const DEFAULT_ROOT_DIR: &str = "./Comet";
const DEFAULT_ASSETS_URL: &str = "https://resources.download.minecraft.net/";
const DEFAULT_MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest.json";

#[derive(typed_builder::TypedBuilder, Default)]
pub struct LauncherConfig {
  #[builder(default = DEFAULT_LAUNCHER_NAME.into(), setter(into))]
  pub(super) name: String,
  #[builder(default = DEFAULT_LAUNCHER_VERSION.into(), setter(into))]
  pub(super) version: String,
  #[builder(default = DEFAULT_ROOT_DIR.into(), setter(into))]
  pub(super) root_dir: PathBuf,
  #[builder(default = DEFAULT_ASSETS_URL.into(), setter(into))]
  pub(super) assets_url: String,
  #[builder(default = DEFAULT_MANIFEST_URL.into(), setter(into))]
  pub(super) manifest_url: String,
  #[builder(default)]
  pub(super) net_s: Option<Arc<Semaphore>>,
  #[builder(default)]
  pub(super) io_s: Option<Arc<Semaphore>>,
}
