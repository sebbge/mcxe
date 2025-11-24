use super::{super::TaskManager, LaunchConfig, Process};

use crate::{
  Instance,
  meta::{ProcessedLibrary, ProcessedVersionMeta},
};
use std::collections::HashMap;
use tokio::process::Command;

pub(super) async fn build(
  instance: Instance,
  process: &TaskManager<Process>,
  meta: ProcessedVersionMeta,
  config: &LaunchConfig,
) -> crate::Result<Command> {
  let java_path = if let Some(custom) = &config.custom_java {
    custom.clone()
  } else {
    let (path, _) = instance.java.get_java(&meta.java_version)?;
    path.ok_or_else(|| crate::Error::Launcher("Java is unavailable".into()))?
  };

  for native in meta.libraries.iter().filter(|l| l.is_native && !l.must_skip) {
    let _io_permit = instance.io_s.clone().acquire_owned().await.unwrap();
    instance.library.extract_native(&config.id, native).await?;
  }

  process.start().await;

  if config.token.is_some() {
    tracing::info!("Session found. Minecraft will run in online mode");
  } else {
    tracing::info!("NO SESSION FOUND. Minecraft will run in offline mode!");
  }

  let (classpath, _) = build_classpath(&instance, &meta);

  let jvm_args = build_jvm_args(&instance, &meta, &classpath, config);
  let game_args = build_game_args(&instance, &meta, config);
  let args = [jvm_args, game_args].concat();

  let mut command = Command::new(java_path);

  #[cfg(windows)]
  command.creation_flags(0x08000000);

  command.args(args);

  Ok(command)
}

fn build_classpath(instance: &Instance, meta: &ProcessedVersionMeta) -> (String, Vec<ProcessedLibrary>) {
  let client_id = meta.inherits_from.as_ref().unwrap_or(&meta.id); // parent version id
  let client_path = instance.client.get_path(client_id).to_string_lossy().to_string();
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
    .get_natives_dir(&config.id)
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

  let java_lib_arg = format!("-Djava.library.path={}", natives_dir);
  if !args.contains(&java_lib_arg) {
    args.push(java_lib_arg);
  }

  if !args.contains(&String::from("-cp"))
    && let Some(cp) = variables.remove("classpath")
  {
    args.push(String::from("-cp"));
    args.push(cp.to_string());
  }

  let jna_tmp_arg = format!("-Djna.tmpdir={}", crate::platform::tmp_dir());
  if !args.contains(&jna_tmp_arg) {
    args.push(jna_tmp_arg);
  }

  args.push(meta.main_class.clone());

  args
}

fn build_game_args(instance: &Instance, meta: &ProcessedVersionMeta, config: &LaunchConfig) -> Vec<String> {
  let token = config.token.clone().unwrap_or_else(|| String::from("0"));
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
    ("version_type", "Blazar"),
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
      re.replace_all(&arg, |caps: &regex::Captures| variables.get(&caps[1]).unwrap_or(&""))
        .to_string()
    })
    .collect()
}
