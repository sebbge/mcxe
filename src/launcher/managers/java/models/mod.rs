use std::path::PathBuf;

pub struct JavaInstallation {
  name: String,
  java_version: u16,
  path_to_executable: PathBuf,
}
