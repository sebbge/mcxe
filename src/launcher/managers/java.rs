use crate::meta::JavaVersion;

use serde::Deserialize;
use std::env::consts::{ARCH, OS};
use std::path::Path;
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Deserialize)]
pub struct ZuluPackage {
  // arch: String,
  pub ext: String,
  // java_version: Vec<u32>,
  // latest: bool,
  // md5_hash: String,
  // name: String,
  // os: String,
  // sha256_hash: String,
  pub size: u64,
  // support_term: String,
  pub url: String,
}

pub const DEFAULT_JAVA_VERSION: u16 = 8;

pub struct JavaManager {
  pub java_dir: PathBuf,
}
impl JavaManager {
  pub fn new(java_dir: PathBuf) -> Self {
    Self { java_dir }
  }
  //
  pub async fn get_java(&self, version: &Option<JavaVersion>) -> crate::Result<(PathBuf, u16)> {
    let major_version = version.as_ref().map(|j| j.major_version).unwrap_or(DEFAULT_JAVA_VERSION);
    let java_path = self.get_java_path(&major_version);

    if !self.test_java(&java_path).await? {
      let e = std::io::Error::other("Java is unavailable");
      return Err(crate::fs::IoError::from_io(e, &java_path).into());
    };

    Ok((java_path, major_version))
  }

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

  async fn test_java(&self, java_path: &Path) -> crate::Result<bool> {
    if !java_path.exists() {
      return Ok(false);
    }

    let output = Command::new(java_path)
      .arg("-version")
      .output()
      .await
      .map_err(|e| crate::Error::Io(crate::fs::IoError::from_io(e, java_path)))?;

    Ok(output.status.success())
  }

  pub async fn fetch_java_package(&self, java_version: &u16) -> crate::Result<ZuluPackage> {
    let ext = "zip";
    let zulu_url = format!(
      "https://api.azul.com/zulu/download/community/v1.0/bundles/latest/?java_version={java_version}&bundle_type=jre&javafx=false&os={OS}&arch={ARCH}&ext={ext}"
    );

    let package_bytes = crate::net::get_bytes(&zulu_url).await?;
    let package = serde_json::from_slice::<ZuluPackage>(&package_bytes)?;
    Ok(package)
  }
}
