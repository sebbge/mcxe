use std::path::PathBuf;

#[derive(typed_builder::TypedBuilder)]
pub struct LaunchConfig {
  #[builder(setter(into))]
  pub(super) id: String,
  #[builder(setter(into))]
  pub(super) version: String,
  #[builder(setter(into))]
  pub(super) username: String,
  #[builder(setter(into))]
  pub(super) uuid: String,
  #[builder(default, setter(strip_option))]
  pub(super) token: Option<String>,
  #[builder(setter(into))]
  pub(super) directory: String,
  #[builder(default, setter(strip_option))]
  pub(super) custom_java: Option<PathBuf>,
  #[builder(default, setter(strip_option))]
  pub(super) extra_jvm_args: Option<Vec<String>>,
  #[builder(default, setter(strip_option))]
  pub(super) extra_game_args: Option<Vec<String>>,
  #[builder(default)]
  #[allow(dead_code)] // TODO: use check_assets field
  pub(super) check_assets: Option<bool>,
}

pub struct LaunchHandle {
  pub id: String,
  pub child: std::process::Child, // TODO: set to tokio process
}
