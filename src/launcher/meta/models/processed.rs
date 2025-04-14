use super::{
  AssetIndexFile, File, LibraryExtract, Logging,
  vanilla::{ClientDownloads, JavaVersion},
  version::VersionType,
};

#[derive(Debug)]
pub struct ProcessedVersionMeta {
  pub id: String,
  pub type_: VersionType,
  pub asset_index: AssetIndexFile, // asset index download
  pub assets: String,              // asset index id
  pub java_version: Option<JavaVersion>,
  pub compliance_level: Option<u16>, // player safety features
  pub downloads: ClientDownloads,
  pub main_class: String,
  pub arguments: ProcessedArguments,
  pub libraries: Vec<ProcessedLibrary>,
  pub minimum_launcher_version: u16,
  pub release_time: String,
  pub time: String,
  pub logging: Option<Logging>,
}

// Processed arguments
#[derive(Debug)]
pub struct ProcessedArguments {}

/// Processed Library
#[derive(Debug)]
pub struct ProcessedLibrary {
  /// Name identifier of the library  
  /// (`<group>:<artifact>:<version>:<classifier>`).
  pub name: String,

  /// Relative path to the file inside the libraries directory.
  pub path: String,

  /// Whether the library should be skipped.
  pub must_skip: bool,

  /// Indicates whether the library is a native (platform-specific) library.
  pub is_native: bool,

  /// Library file metadata.
  ///
  /// - If `is_native` is `false`, the file is downloaded and added to the classpath.
  /// - If `true`, this file must be downloaded and extracted
  ///   to the version's `natives` directory before game launch.
  pub file: Option<File>,

  /// Optional extraction rules for native libraries.  
  /// Only relevant when `is_native` is `true`.  
  /// Specifies which files or directories should be **excluded** during extraction.
  /// (usually the `META-INF/` folder).
  pub extract: Option<LibraryExtract>,
}
