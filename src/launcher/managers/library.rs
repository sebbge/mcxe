use crate::meta::ProcessedLibrary;
use std::{io::Read, path::PathBuf};

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

  pub fn get_version_natives_dir(&self, version_id: &str) -> PathBuf {
    self.natives_dir.join(version_id)
  }
  //

  pub async fn extract_native(&self, version_id: &str, native: &ProcessedLibrary) -> crate::Result<()> {
    let natives_dir = self.get_version_natives_dir(version_id);
    let path = self.get_lib_path(native);

    crate::io::zip::for_each(&path, |mut file: crate::io::zip::ZipFile| {
      let name = file.name();

      if name.starts_with("META-INF/") || name.ends_with('/') {
        return Ok(());
      }

      let filename = name.rsplit('/').next().unwrap_or_default();

      let native_path = natives_dir.join(filename);

      let mut bytes = Vec::with_capacity(file.size() as usize);
      file
        .read_to_end(&mut bytes)
        .map_err(|e| crate::io::IOError::from_io(e, &path))?;

      crate::io::write_sync(&native_path, &bytes)?;

      Ok(())
    })?;

    Ok(())
  }
}
