use crate::meta::{File, ProcessedLibrary};
use std::{io::Read, path::PathBuf};

pub struct LibraryManager {
  pub libraries_dir: PathBuf,
  pub natives_dir: PathBuf,
  pub client_dir: PathBuf,
}
impl LibraryManager {
  pub fn new(libraries_dir: PathBuf, natives_dir: PathBuf, client_dir: PathBuf) -> Self {
    Self {
      libraries_dir,
      natives_dir,
      client_dir,
    }
  }

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
  pub async fn get_client_download_size(&self, client: &File, version_id: &str) -> u64 {
    let client_path = self.get_client_path(version_id);
    if !client_path.exists() { client.size } else { 0 }
  }

  pub fn get_lib_path(&self, lib: &ProcessedLibrary) -> PathBuf {
    self.libraries_dir.join(&lib.path)
  }

  pub fn get_client_path(&self, version_id: &str) -> PathBuf {
    self.client_dir.join(version_id).join(format!("{}.jar", version_id))
  }

  pub fn get_natives_dir(&self, instance_id: &str) -> PathBuf {
    self.natives_dir.join(instance_id)
  }

  pub async fn extract_native(&self, version_id: &str, native: &ProcessedLibrary) -> crate::Result<()> {
    let natives_dir = self.get_natives_dir(version_id);
    let path = self.get_lib_path(native);
    if !path.exists() {
      return Ok(());
    }

    crate::fs::zip::for_each(&path, |mut file: crate::fs::zip::ZipFile| {
      let name = file.name();

      if name.starts_with("META-INF/") || name.ends_with('/') {
        return Ok(());
      }

      let filename = name.rsplit('/').next().unwrap_or_default();

      let native_path = natives_dir.join(filename);

      let mut bytes = Vec::with_capacity(file.size() as usize);
      file.read_to_end(&mut bytes).map_err(|e| crate::fs::IoError::from_io(e, &path))?;

      crate::fs::write_sync(&native_path, &bytes)?;

      Ok(())
    })?;

    Ok(())
  }
}
