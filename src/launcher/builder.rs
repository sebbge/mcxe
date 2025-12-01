use std::{path::PathBuf, sync::Arc};
use tokio::sync::Semaphore;

#[derive(typed_builder::TypedBuilder, Default)]
pub struct LauncherConfig {
  #[builder(setter(into))]
  pub(super) name: String,
  #[builder(setter(into))]
  pub(super) version: String,
  #[builder(setter(into))]
  pub(super) root_dir: PathBuf,
  #[builder(setter(into))]
  pub(super) assets_url: String,
  #[builder(setter(into))]
  pub(super) manifest_url: String,
  #[builder(default)]
  pub(super) net_s: Option<Arc<Semaphore>>,
  #[builder(default)]
  pub(super) io_s: Option<Arc<Semaphore>>,
}
