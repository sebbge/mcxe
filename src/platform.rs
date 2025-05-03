use std::{
  env::consts::{ARCH, OS},
  sync::OnceLock,
};

static VALID_CLASSIFIERS: OnceLock<Vec<String>> = OnceLock::new();
static CLASSPATH_SEPARATOR: OnceLock<&str> = OnceLock::new();

pub fn native_classifier() -> &'static Vec<String> {
  VALID_CLASSIFIERS.get_or_init(|| {
    let mut classifiers = Vec::new();
    classifiers.push(format!("natives-{}", OS));

    //todo: improve this
    match (OS, ARCH) {
      ("macos", "aarch64") => {
        classifiers.push("natives-macos-arm64".into());
        classifiers.push("natives-osx".into());
      }
      ("windows", "x86") => {
        classifiers.push("natives-windows-x86".into());
        classifiers.push("natives-windows-32".into());
      }
      ("windows", "aarch64") | ("windows", "arm") => {
        classifiers.push("natives-windows-arm64".into());
        classifiers.push("natives-windows-64".into());
      }
      ("windows", "x86_64") => {
        classifiers.push("natives-windows-64".into());
      }
      ("linux", "x86_64") => {
        classifiers.push("linux-x86_64".into());
      }
      ("linux", "aarch64") => {
        classifiers.push("linux-aarch_64".into());
      }
      _ => {}
    }

    classifiers
  })
}

pub fn classpath_separator() -> &'static str {
  CLASSPATH_SEPARATOR.get_or_init(|| if OS == "windows" { ";" } else { ":" })
}

pub fn tmp_dir() -> String {
  let tmp_dir = std::env::temp_dir();
  crate::io::create_dir_all_sync(&tmp_dir).ok();
  tmp_dir.to_string_lossy().to_string()
}
