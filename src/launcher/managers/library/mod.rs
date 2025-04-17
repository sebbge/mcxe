use crate::meta::ProcessedLibrary;
use std::path::PathBuf;

pub struct LibraryManager {
  pub libraries_dir: PathBuf,
  pub natives_dir: PathBuf,
}
impl LibraryManager {
  pub fn new(libraries_dir: PathBuf, natives_dir: PathBuf) -> Self {
    Self {
      libraries_dir,
      natives_dir,
    }
  }
  //

  pub async fn get_download_size(&self, libraries: &[ProcessedLibrary]) -> u64 {
    let mut total = 0;

    for lib in libraries {
      if lib.must_skip {
        continue;
      }

      let lib_path = self.libraries_dir.join(&lib.path);
      if !lib_path.exists() {
        total += lib.size.unwrap_or(0);
      }
    }

    total
  }

  pub fn get_lib_path(&self, lib: &ProcessedLibrary) -> PathBuf {
    self.libraries_dir.join(&lib.path)
  }
  //

  pub async fn extract_native(&self, native: &ProcessedLibrary) -> crate::Result<()> {
    let native_path = self.get_lib_path(native);

    //TODO

    Ok(())
  }
}
