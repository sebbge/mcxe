use super::{File, Rule};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Library {
  pub downloads: Option<LibraryDownloads>,
  pub name: String,
  pub rules: Option<Vec<Rule>>,

  /// extraction rules for native libraries.  
  pub extract: Option<LibraryExtract>,

  /// OS guide for classifier keys
  pub natives: Option<LibraryNatives>,
}

#[derive(Deserialize, Debug)]
pub struct LibraryDownloads {
  pub artifact: Option<File>,

  // Keys can be:
  // linux-x86_64
  // natives-linux
  // natives-osx
  // natives-windows
  // natives-macos
  // natives-windows-32
  // natives-windows-64
  pub classifiers: HashMap<String, File>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LibraryExtract {
  pub exclude: Vec<String>, // always ["META-INF/"]
}

// Natives
#[derive(Deserialize, Debug)]
pub struct LibraryNatives {
  // Can be
  // "linux-x86_64"
  // "natives-linux"
  pub linux: Option<String>,

  // Can be
  // "natives-macos"
  // "natives-osx"
  pub osx: Option<String>,

  // Can be
  // "natives-windows"
  // "natives-windows-${arch}"
  pub windows: Option<String>,
}

// #[derive(Deserialize, Debug)]
// #[serde(rename_all = "kebab-case")]
// pub struct Classifiers {
//   pub natives_linux: Option<File>,
//   pub natives_osx: Option<File>,
//   pub natives_windows: Option<File>,
//   pub natives_macos: Option<File>,
//   pub natives_windows_32: Option<File>,
//   pub natives_windows_64: Option<File>,

//   #[serde(rename = "linux-x86_64")]
//   pub linux_x86_64: Option<File>,
// }
