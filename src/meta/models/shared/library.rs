use super::{File, Rule};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Library {
  pub downloads: Option<LibraryDownloads>,
  pub name: String,
  pub rules: Option<Vec<Rule>>,

  // extraction rules for native libraries.
  // pub extract: Option<LibraryExtract>,

  // OS guide for classifier keys
  // pub natives: Option<LibraryNatives>,

  // fabric-like libraries
  pub sha1: Option<String>,
  pub url: Option<String>,
  pub size: Option<u64>,
}

#[derive(Deserialize, Debug)]
pub struct LibraryDownloads {
  pub artifact: Option<File>,

  // linux-x86_64
  // natives-linux
  // natives-macos
  // natives-osx
  // natives-windows
  // natives-windows-32
  // natives-windows-64
  #[serde(default)]
  pub classifiers: HashMap<String, File>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct LibraryExtract {
  pub exclude: Vec<String>, // always ["META-INF/"]
}

// Natives
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct LibraryNatives {
  // "linux-x86_64"
  // "natives-linux"
  pub linux: Option<String>,

  // "natives-macos"
  // "natives-osx"
  pub osx: Option<String>,

  // "natives-windows"
  // "natives-windows-${arch}"
  pub windows: Option<String>,
}
