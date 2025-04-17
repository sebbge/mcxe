mod models;
pub use models::*;

use std::{collections::HashMap, path::PathBuf};

pub struct JavaManager {
  pub java_dir: PathBuf,
  java_installations: HashMap<String, JavaInstallation>,
}
impl JavaManager {
  pub fn new(java_dir: PathBuf) -> Self {
    let java_installations = HashMap::new();

    Self {
      java_dir,
      java_installations,
    }
  }
}
