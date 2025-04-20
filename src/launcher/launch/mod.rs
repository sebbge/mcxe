mod models;
pub use models::*;

use crate::{
  Instance,
  meta::{ProcessedLibrary, ProcessedVersionMeta},
};
use std::collections::HashMap;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};

pub(super) async fn launch_game(instance: Instance, config: LaunchConfig) -> crate::Result<LaunchHandle> {
  let meta = instance.version.read().await.meta(&config.version).await?;

  for native in meta.libraries.iter().filter(|l| l.is_native && !l.must_skip) {
    let _io_permit = instance.io_s.clone().acquire_owned().await.unwrap();
    instance.library.extract_native(&meta.id, native).await?;
  }

  // meta.checksum()

  let java_path = if let Some(custom) = &config.custom_java {
    custom.clone()
  } else {
    // check java existence
    std::path::PathBuf::new()
  };

  if config.token.is_some() {
    // info!("LAUNCH", "Session found. Minecraft will run in online mode");
  } else {
    // info!("LAUNCH", "NO SESSION FOUND. Minecraft will run in OFFLINE MODE!");
  }

  let (classpath, skipped_libs) = build_classpath(&instance, &meta);
  println!("Skipped {} libraries.", skipped_libs.len());

  let jvm_args = build_jvm_args(&instance, &meta, &classpath, &config);
  let game_args = build_game_args(&instance, &meta, &config);
  let args = [jvm_args, game_args].concat();

  let mut command = Command::new(java_path);

  #[cfg(windows)]
  command.creation_flags(0x08000000);

  let child = command
    .args(args)
    .stdout(Stdio::piped()) // mc logs
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|e| crate::Error::Launcher(format!("Failed to execute Minecraft process: {}", e)))?;

  Ok(LaunchHandle { id: config.id, child })
}

fn build_classpath(instance: &Instance, meta: &ProcessedVersionMeta) -> (String, Vec<ProcessedLibrary>) {
  let client_path = instance.client.get_path(&meta.id).to_string_lossy().to_string();
  let separator = crate::platform::classpath_separator();

  let mut skipped_libs = Vec::new();

  let mut classpath = Vec::new();
  for lib in &meta.libraries {
    if lib.must_skip {
      skipped_libs.push(lib.clone());
      continue;
    }

    let path = instance.library.get_lib_path(lib).to_string_lossy().to_string();

    if !classpath.contains(&path) {
      classpath.push(path);
    }
  }

  classpath.push(client_path);
  (classpath.join(separator), skipped_libs)
}

fn build_jvm_args(
  instance: &Instance,
  meta: &ProcessedVersionMeta,
  classpath: &str,
  config: &LaunchConfig,
) -> Vec<String> {
  let natives_dir = instance
    .library
    .get_version_natives_dir(&meta.id)
    .to_string_lossy()
    .to_string();
  let libraries_dir = instance.library.libraries_dir.to_string_lossy().to_string();
  let separator = crate::platform::classpath_separator().to_string();
  let mut variables: HashMap<&str, &str> = HashMap::from([
    // Minecraft
    ("natives_directory", natives_dir.as_str()),
    ("launcher_name", &instance.launcher_name),
    ("launcher_version", &instance.launcher_version),
    ("classpath", classpath),
    // External
    ("version_name", &meta.id),
    ("library_directory", &libraries_dir),
    ("classpath_separator", &separator),
  ]);

  let args = if let Some(extra) = &config.extra_jvm_args {
    [meta.arguments.jvm.clone(), extra.clone()].concat()
  } else {
    meta.arguments.jvm.clone()
  };

  let mut args = replace_placeholders(args, &mut variables);

  if let Some(natives_dir) = variables.remove("natives_directory") {
    args.push(format!("-Djava.library.path={}", natives_dir));
  }
  if let Some(cp) = variables.remove("classpath") {
    args.push(String::from("-cp"));
    args.push(cp.to_string());
  }

  args.push(meta.main_class.clone());

  args
}

fn build_game_args(instance: &Instance, meta: &ProcessedVersionMeta, config: &LaunchConfig) -> Vec<String> {
  let token = config.token.clone().unwrap_or_else(|| String::from("none"));
  let game_dir = instance.games_dir.join(&config.directory).to_string_lossy().to_string();
  let assets_dir = instance.asset.assets_dir.to_string_lossy().to_string();

  let mut variables = HashMap::from([
    // Minecraft
    ("auth_player_name", config.username.as_str()),
    ("version_name", &meta.id),
    ("game_directory", &game_dir),
    ("assets_root", &assets_dir),
    ("assets_index_name", &meta.asset_index.id),
    ("auth_uuid", &config.uuid),
    ("auth_session", token.as_ref()), // for older versions
    ("auth_access_token", token.as_ref()),
    ("user_type", "msa"),
    ("version_type", "Pandax"),
    // External
  ]);

  let args = if let Some(extra) = &config.extra_game_args {
    [meta.arguments.game.clone(), extra.clone()].concat()
  } else {
    meta.arguments.game.clone()
  };

  replace_placeholders(args, &mut variables)
}

fn replace_placeholders(args: Vec<String>, variables: &mut HashMap<&str, &str>) -> Vec<String> {
  let re = regex::Regex::new(r"\$\{([^}]+)\}").unwrap();

  args
    .into_iter()
    .map(|arg| {
      re.replace_all(&arg, |caps: &regex::Captures| variables.remove(&caps[1]).unwrap_or(""))
        .to_string()
    })
    .collect()
}
