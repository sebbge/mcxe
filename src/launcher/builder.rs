use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::Semaphore;

#[derive(typed_builder::TypedBuilder, Default)]
pub struct LauncherConfig {
  #[builder(setter(into))]
  pub(super) name: String,
  #[builder(setter(into))]
  pub(super) version: String,
  #[builder(setter(into))]
  pub(super) root_dir: PathBuf,
  #[builder(default)]
  pub(super) games_dir: Option<PathBuf>,
  #[builder(default)]
  pub(super) java_dir: Option<PathBuf>,
  #[builder(default)]
  pub(super) versions_dir: Option<PathBuf>,
  #[builder(default)]
  pub(super) libraries_dir: Option<PathBuf>,
  #[builder(default)]
  pub(super) natives_dir: Option<PathBuf>,
  #[builder(setter(into))]
  pub(super) assets_url: String,
  #[builder(setter(into))]
  pub(super) manifest_url: String,
  #[builder(default)]
  pub(super) manifest_path: Option<PathBuf>,
  #[builder(default)]
  pub(super) manifest_refresh: Option<Duration>,
  #[builder(default)]
  pub(super) assets_dir: Option<PathBuf>,
  #[builder(default)]
  pub(super) net_s: Option<Arc<Semaphore>>,
  #[builder(default)]
  pub(super) io_s: Option<Arc<Semaphore>>,
}
