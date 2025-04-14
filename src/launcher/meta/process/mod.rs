use super::models::{
  Action, Library,
  child::ChildVersionMeta,
  processed::{ProcessedLibrary, ProcessedVersionMeta},
  vanilla::FullVersionMeta,
};
use std::collections::HashMap;

impl FullVersionMeta {
  pub fn process(self) -> ProcessedVersionMeta {
    ProcessedVersionMeta::process(self);
  }
  pub fn merge_and_process(self, child_meta: ChildVersionMeta) {
    ProcessedVersionMeta::merge(self, child_meta);
  }
}

impl ChildVersionMeta {
  pub fn merge_and_process(self, full_meta: FullVersionMeta) {
    ProcessedVersionMeta::merge(full_meta, self);
  }
}

impl ProcessedVersionMeta {
  pub fn merge(parent: FullVersionMeta, child: ChildVersionMeta) -> FullVersionMeta {
    FullVersionMeta {
      id: child.id,
      type_: child.type_,
      asset_index: parent.asset_index,
      assets: parent.assets,
      java_version: parent.java_version,
      compliance_level: parent.compliance_level,
      downloads: parent.downloads,
      main_class: child.main_class.unwrap_or(parent.main_class),
      arguments: (),
      minecraft_arguments: match (parent.minecraft_arguments, child.minecraft_arguments) {
        (Some(p), Some(c)) => Some(format!("{} {}", p, c)),
        (Some(p), None) => Some(p),
        (None, Some(c)) => Some(c),
        (None, None) => None,
      },
      libraries: [parent.libraries, child.libraries].concat(),
      minimum_launcher_version: parent.minimum_launcher_version,
      release_time: child.release_time,
      time: child.release_time,
      logging: parent.logging,
    }
  }
  pub fn process(meta: FullVersionMeta) {}
}

impl Library {
  fn parse_lib_name(&self) -> (String, String, Option<String>) {
    let parts: Vec<&str> = self.name.split(':').collect();
    let name = parts[0..2].join(":");
    let version = parts[2].to_string();
    let classifier = if parts.len() > 3 {
      Some(parts[3].to_string())
    } else {
      None
    };
    (name, version, classifier)
  }

  /// Returns `true` if the library must be skipped, `false` otherwise.
  fn process_rules(&self) -> bool {
    let rules = match &self.rules {
      Some(r) => r,
      None => return false,
    };

    let mut is_allowed = false;

    for rule in rules {
      let allow = rule.action == Action::Allow;

      let mut applies = true;

      if let Some(os_rule) = &rule.os {
        if let Some(os) = &os_rule.name {
          if os.as_str() != std::env::consts::OS {
            applies = false;
          }
        }
        if let Some(arch) = &os_rule.arch {
          if arch != std::env::consts::ARCH {
            applies = false;
          }
        }
      }

      if applies {
        if allow {
          is_allowed = true;
        } else {
          return true;
        }
      }
    }

    !is_allowed
  }

  fn get_path_from_artifact(&self) -> crate::Result<String> {
    let parts: Vec<&str> = self.name.split(':').collect();

    if parts.len() < 3 {
      return Err(crate::Error::Metadata(format!(
        "Invalid library name format: {}",
        &self.name
      )));
    }

    let (group, name, version, classifier_ext) = match parts.as_slice() {
      [g, n, v] => (*g, *n, *v, None),
      [g, n, v, c] => (*g, *n, *v, Some(*c)),
      _ => {
        return Err(crate::Error::Metadata(format!(
          "Invalid library name format: {}",
          &self.name
        )));
      }
    };

    let (classifier, ext) = if let Some(raw) = classifier_ext {
      let mut split = raw.split('@');
      (split.next(), split.next().unwrap_or("jar"))
    } else {
      (None, "jar")
    };

    let path = match classifier {
      Some(c) => format!(
        "{}/{}/{}/{}-{}-{}.{}",
        group.replace('.', "/"),
        name,
        version,
        name,
        version,
        c,
        ext
      ),
      None => format!(
        "{}/{}/{}/{}-{}.{}",
        group.replace('.', "/"),
        name,
        version,
        name,
        version,
        ext
      ),
    };

    Ok(path)
  }
}

pub fn deduplicate_libraries(libraries: Vec<Library>) -> Vec<Library> {
  let mut map: HashMap<(String, Option<String>), Library> = HashMap::new();

  for lib in libraries {
    let (name, version, classifier) = lib.parse_lib_name();
    let key = (name, classifier);

    match map.get(&key) {
      Some(existing_lib) => {
        let (_, existing_version, _) = existing_lib.parse_lib_name();
        if version > existing_version {
          map.insert(key, lib);
        }
      }
      None => {
        map.insert(key, lib);
      }
    }
  }

  map.into_values().collect()
}

impl From<Library> for Vec<ProcessedLibrary> {
  fn from(lib: Library) -> Self {
    let must_skip = lib.process_rules();

    let mut libs = Vec::new();

    if let Some(downloads) = &lib.downloads {
      if let Some(artifact) = &downloads.artifact {
        let path = artifact.path.clone();

        libs.push(ProcessedLibrary {
          name: lib.name.clone(),
          path: path.unwrap_or_else(|| lib.get_path_from_artifact().unwrap_or_default()),
          must_skip,
          is_native: false,
          file: Some(artifact.clone()),
          extract: None,
        });
      }

      if !downloads.classifiers.is_empty() {
        let os_keys: [&str; 3] = if let Some(natives_map) = &lib.natives {
          [
            natives_map.windows.as_deref().unwrap_or("natives-windows"),
            natives_map.linux.as_deref().unwrap_or("natives-linux"),
            natives_map.osx.as_deref().unwrap_or("natives-osx"),
          ]
        } else {
          ["natives-windows", "natives-linux", "natives-osx"]
        };

        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        let arch_suffix = match arch {
          "x86_64" => "64",
          "x86" => "32",
          _ => arch,
        };

        let expected_keys: Vec<String> = match os {
          "windows" => vec!["natives-windows".into(), format!("natives-windows-{}", arch_suffix)],
          "linux" => vec!["natives-linux".into()],
          "macos" => vec!["natives-osx".into(), "natives-macos".into()],
          _ => vec![],
        };

        for &key in &os_keys {
          if let Some(file) = downloads.classifiers.get(key) {
            let path = file.path.clone();

            let must_skip = !expected_keys.iter().any(|expected| expected == key);

            libs.push(ProcessedLibrary {
              name: lib.name.clone(),
              path: path.unwrap_or_else(|| lib.get_path_from_artifact().unwrap_or_default()),
              must_skip,
              is_native: true,
              file: Some(file.clone()),
              extract: lib.extract.clone(),
            });
          }
        }
      }
    }

    libs
  }
}
