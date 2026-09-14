mod models;
pub use models::*;

pub mod platform;

use std::collections::HashMap;
use url::Url;

// ===> Public wrappers
impl FullVersionMeta {
  /// Processes this full version metadata into a `ProcessedVersionMeta`.
  pub fn process(self) -> ProcessedVersionMeta {
    ProcessedVersionMeta::process(self, None)
  }

  /// Merges this full version with a child version and processes the result.
  pub fn merge_and_process(self, child_meta: PartialVersionMeta) -> ProcessedVersionMeta {
    ProcessedVersionMeta::merge_and_process(self, child_meta)
  }
}

// ===> Merging and processing
impl ProcessedVersionMeta {
  /// Merges a full version with a child version, producing a unified `ProcessedVersionMeta`.
  fn merge_and_process(mut parent: FullVersionMeta, child: PartialVersionMeta) -> ProcessedVersionMeta {
    parent.libraries.extend(child.libraries);
    parent.deduplicate_libraries().ok();

    let arguments = match (parent.arguments, child.arguments) {
      (Some(mut a), Some(b)) => {
        a.game.extend(b.game);
        a.jvm.extend(b.jvm);
        Some(a)
      }
      (Some(a), None) | (None, Some(a)) => Some(a),
      (None, None) => None,
    };

    let minecraft_arguments = match (parent.minecraft_arguments, child.minecraft_arguments) {
      (Some(p), Some(c)) => Some(format!("{} {}", p, c)),
      (Some(p), None) => Some(p),
      (None, Some(c)) => Some(c),
      (None, None) => None,
    };

    let full_meta = FullVersionMeta {
      id: child.id,
      type_: child.type_,
      asset_index: parent.asset_index,
      assets: parent.assets,
      java_version: parent.java_version,
      compliance_level: Some(0),
      downloads: parent.downloads,
      main_class: child.main_class.unwrap_or(parent.main_class),
      arguments,
      minecraft_arguments,
      libraries: parent.libraries,
      minimum_launcher_version: parent.minimum_launcher_version,
      release_time: child.release_time,
      time: child.time,
      logging: parent.logging,
    };

    ProcessedVersionMeta::process(full_meta, Some(parent.id))
  }

  /// Processes a `FullVersionMeta` into a `ProcessedVersionMeta`.
  fn process(meta: FullVersionMeta, parent_id: Option<String>) -> ProcessedVersionMeta {
    let libraries = meta.libraries.into_iter().flat_map(|lib| lib.process_library()).collect::<Vec<ProcessedLibrary>>();

    let mut arguments = meta.arguments.map(|args| args.process_arguments()).unwrap_or_default();

    if let Some(minecraft_args) = meta.minecraft_arguments {
      let legacy_args = minecraft_args.split_whitespace().map(String::from);
      arguments.game.extend(legacy_args);
    }

    ProcessedVersionMeta {
      id: meta.id,
      inherits_from: parent_id,
      type_: meta.type_,
      asset_index: meta.asset_index,
      assets: meta.assets,
      java_version: meta.java_version,
      compliance_level: meta.compliance_level,
      downloads: meta.downloads,
      main_class: meta.main_class,
      arguments,
      libraries,
      minimum_launcher_version: meta.minimum_launcher_version,
      release_time: meta.release_time,
      time: meta.time,
      logging: meta.logging,
    }
  }
}

//* ==ARGUMENTS & LIBRARIES PROCESSING== *//

//===> RULE PROCESSING
impl Rule {
  /// Evaluates the rule and returns `Some(Action::Allow)` or `Some(Action::Disallow)`
  /// if the rule applies to this system. Returns `None` if the rule is not relevant.
  fn evaluate(&self) -> Option<Action> {
    if let Some(os_rule) = &self.os
      && let Some(os_name) = &os_rule.name
      && os_name.as_str() != std::env::consts::OS
    {
      return None;
    }

    if let Some(_feature_rule) = &self.features {
      //- Features of the official Minecraft Launcher
      //- Not supported here atm
      return None;
    }

    Some(self.action.clone())
  }
}
/// Trait for ruled items that needs to be processed (libraries & arguments)
trait RuledItem {
  fn get_rules(&self) -> Option<&Vec<Rule>>;

  /// Returns `true` if the item must be skipped, `false` otherwise.
  fn process_rules(&self) -> bool {
    let rules = match self.get_rules() {
      Some(r) => r,
      None => return false, // No rules = include by default
    };

    let mut action = Action::Disallow; // Disallow by default

    for rule in rules {
      if let Some(rule_action) = rule.evaluate() {
        action = rule_action // Rule is applicable; update the action.
      } // If rule is not applicable on this system; ignore.
    }

    !action.into_bool()
  }
}

//===> LIBRARIES
impl RuledItem for Library {
  fn get_rules(&self) -> Option<&Vec<Rule>> {
    self.rules.as_ref()
  }
}

impl Library {
  /// Processes a library by evaluating its rules and transforming it into one or more
  /// `ProcessedLibrary` instances ready for download or launch.
  ///
  /// - Determines whether the library should be skipped by evaluating its rule set (`must_skip`).
  /// - Nested native classifiers are converted to a separate `ProcessedLibrary` with the `is_native` flag set.
  fn process_library(self) -> Vec<ProcessedLibrary> {
    let must_skip = self.process_rules();

    let mut libs = Vec::new();

    let fallback_path = self.get_path_from_name().unwrap_or_default();

    if let Some(downloads) = self.downloads {
      //= Main artifact
      if let Some(artifact) = downloads.artifact {
        let path = artifact.path.clone();
        let is_native = self.name.contains("natives"); //- +1.19 natives

        libs.push(ProcessedLibrary {
          name: self.name.clone(),
          path: path.unwrap_or(fallback_path),
          url: Some(artifact.url),
          sha1: Some(artifact.sha1),
          size: Some(artifact.size),
          must_skip: if is_native && !must_skip {
            Library::must_skip_native(&self.name)
          } else {
            must_skip
          },
          is_native,
        });
      }

      //= Native classifiers
      //- Note: pre-1.19 native libraries were included under
      //- the "classifiers" field within the main library entry.
      //- +1.19 format doesn't have this field, native libraries are
      //- defined as separate artifacts with their own library entries
      //- with "natives-{os}" in the name.
      if !downloads.classifiers.is_empty() {
        for (key, lib) in downloads.classifiers.into_iter() {
          let must_skip = Library::must_skip_native(&key);

          //- Classifier files should always contain a "path" field.
          //- Using fallback_path here will place the native library
          //- in the same path as the main artifact, which will cause issues.
          let lib_path = lib.path.unwrap_or_default();

          libs.push(ProcessedLibrary {
            name: self.name.clone(),
            url: Some(lib.url),
            path: lib_path,
            sha1: Some(lib.sha1),
            size: Some(lib.size),
            must_skip,
            is_native: true,
          });
        }
      }
    } else {
      //- Likely a Fabric-style library without a "downloads" field.
      //- The full URL must be constructed manually using the "url" field as
      //- the base repository URL and appending the library's relative path to it.
      let lib_url = self
        .url
        .as_ref()
        .and_then(|repo_url| Url::parse(repo_url).ok())
        .and_then(|base| base.join(&fallback_path).ok())
        .map(|url| url.to_string());

      libs.push(ProcessedLibrary {
        name: self.name,
        path: fallback_path,
        url: lib_url,
        sha1: self.sha1,
        size: self.size,
        must_skip,
        is_native: false,
      });
    }

    libs
  }

  /// Parses the library name into `(group, name, version, classifier, extension)`.
  fn parse_lib_name(&self) -> crate::Result<(String, String, String, Option<String>, String)> {
    let parts: Vec<&str> = self.name.split(':').collect();

    if parts.len() < 3 {
      return Err(crate::Error::Metadata(format!("Invalid library name format: {}", self.name)));
    }

    let (group, name, version, classifier_ext) = match parts.as_slice() {
      [g, n, v] => (g.to_string(), n.to_string(), v.to_string(), None),
      [g, n, v, c] => (g.to_string(), n.to_string(), v.to_string(), Some(c.to_string())),
      _ => {
        return Err(crate::Error::Metadata(format!("Invalid library name format: {}", self.name)));
      }
    };

    let (classifier, ext) = if let Some(raw) = classifier_ext {
      let mut split = raw.split('@');
      (split.next().map(|s| s.to_string()), split.next().unwrap_or("jar").to_string())
    } else {
      (None, "jar".to_string())
    };

    Ok((group, name, version, classifier, ext))
  }

  /// Gets the library relative path from the artifact name.
  fn get_path_from_name(&self) -> crate::Result<String> {
    let (group, name, version, classifier, ext) = self.parse_lib_name()?;

    let path = match classifier {
      Some(c) => format!("{}/{}/{}/{}-{}-{}.{}", group.replace('.', "/"), name, version, name, version, c, ext),
      None => format!("{}/{}/{}/{}-{}.{}", group.replace('.', "/"), name, version, name, version, ext),
    };

    Ok(path)
  }

  /// Determines whether a native library should be skipped based on this system.
  fn must_skip_native(native_lib_name: &str) -> bool {
    let classifiers = crate::meta::platform::native_classifier();
    !classifiers.iter().any(|suffix| native_lib_name.ends_with(suffix))
  }
}

impl FullVersionMeta {
  fn deduplicate_libraries(&mut self) -> crate::Result<()> {
    let mut seen: HashMap<(String, String, Option<String>, String), Library> = HashMap::new();

    for lib in std::mem::take(&mut self.libraries) {
      let (group, name, version, classifier, extension) = lib.parse_lib_name()?;
      let key = (group, name, classifier, extension);

      match seen.get_mut(&key) {
        Some(existing_lib) => {
          let (_, _, existing_version, _, _) = existing_lib.parse_lib_name()?;
          if version > existing_version {
            *existing_lib = lib;
          };
        }
        None => {
          seen.insert(key, lib);
        }
      };
    }

    self.libraries = seen.into_values().collect();

    Ok(())
  }
}

//==> ARGUMENTS
impl RuledItem for RuledArgument {
  fn get_rules(&self) -> Option<&Vec<Rule>> {
    Some(&self.rules)
  }
}

impl Argument {
  fn process_argument(self) -> Option<ArgumentValue> {
    match self {
      Argument::Normal(arg) => Some(arg), // No rules = include by default
      Argument::Ruled(ruled_arg) => {
        let must_skip = ruled_arg.process_rules();
        match must_skip {
          true => None,                   // Skip argument
          false => Some(ruled_arg.value), // Include argument
        }
      }
    }
  }
}

impl Arguments {
  fn process_arguments(self) -> ProcessedArguments {
    let process_args = |args: Vec<Argument>| -> Vec<String> {
      let mut result = Vec::new();
      for argument in args {
        if let Some(value) = argument.process_argument() {
          match value {
            ArgumentValue::Single(arg) => result.push(arg),   // Add single argument
            ArgumentValue::Many(args) => result.extend(args), // Add multiple arguments
          }
        } // If None: Invalid ruled argument
      }
      result
    };

    ProcessedArguments {
      game: process_args(self.game),
      jvm: process_args(self.jvm),
    }
  }
}
