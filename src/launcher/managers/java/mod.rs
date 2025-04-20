mod models;
use models::*;

use std::env::consts::{ARCH, OS};
use std::path::PathBuf;
use std::{path::Path, process::Command};

// TODO: Windows 11 cmd prompt?
// #[cfg(windows)]
// use std::os::windows::process::CommandExt;

pub struct JavaManager {
  pub java_dir: PathBuf,
}
impl JavaManager {
  pub fn new(java_dir: PathBuf) -> Self {
    Self { java_dir }
  }
  //

  fn get_java_path(&self, version: &u16) -> PathBuf {
    let mut java_path = self.java_dir.join(format!("jre-{}", version)).join("bin");

    if cfg!(target_os = "windows") {
      java_path.push("javaw");
      java_path.set_extension("exe");
    } else {
      java_path.push("java");
    }

    java_path
  }

  fn test_java(&self, java_path: &Path) -> crate::Result<bool> {
    if !java_path.exists() {
      return Ok(false);
    }

    let output = Command::new(java_path)
      .arg("-version")
      .output()
      .map_err(|e| crate::Error::Common(crate::io::IOError::from_io(e, java_path).into()))?;

    Ok(output.status.success())
  }

  pub fn get_java(&self, version: &u16) -> crate::Result<Option<PathBuf>> {
    let java_path = self.get_java_path(version);

    if !self.test_java(&java_path)? {
      return Ok(None);
    }

    Ok(Some(java_path))
  }

  pub async fn fetch_java_package(&self, java_version: &u16) -> crate::Result<ZuluPackage> {
    let ext = "zip";
    let zulu_url = format!(
      "https://api.azul.com/zulu/download/community/v1.0/bundles/latest/?java_version={java_version}&bundle_type=jre&javafx=false&os={OS}&arch={ARCH}&ext={ext}"
    ); //??

    let package_bytes = crate::net::get_bytes(&zulu_url).await?;
    let package = serde_json::from_slice::<ZuluPackage>(&package_bytes)?;
    Ok(package)
  }
}
