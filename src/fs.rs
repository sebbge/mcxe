use sha1::Digest;
use std::time::{Duration, SystemTime};
use std::{io::Write, path::Path};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};

pub mod error {
  use std::path::{Path, PathBuf};

  #[derive(thiserror::Error, Debug)]
  #[error("I/O error at '{path}': {message}")]
  pub struct IoError {
    pub(crate) path: PathBuf,
    pub(crate) message: String,
  }

  impl IoError {
    pub fn from_io(e: std::io::Error, path: &Path) -> Self {
      Self {
        path: path.to_path_buf(),
        message: fmt_err_msg(&e),
      }
    }

    pub fn from_zip(e: zip::result::ZipError, path: &Path) -> Self {
      Self {
        path: path.to_path_buf(),
        message: fmt_err_msg(&e),
      }
    }
  }

  fn fmt_err_msg(error: &dyn std::error::Error) -> String {
    let mut message = error.to_string();
    if let Some(source) = error.source() {
      message.push_str(": ");
      message.push_str(&source.to_string());
    }
    message
  }
}

pub use error::IoError;
pub type IoResult<T> = std::result::Result<T, IoError>;

pub mod zip {
  use std::path::Path;

  pub type ZipFile<'a> = zip::read::ZipFile<'a, std::io::Cursor<Vec<u8>>>;

  pub fn for_each<F>(path: impl AsRef<Path>, mut f: F) -> super::IoResult<()>
  where
    F: FnMut(ZipFile) -> super::IoResult<()>,
  {
    let path = path.as_ref();
    let buf = std::fs::read(path).map_err(|e| super::IoError::from_io(e, path))?;
    let reader = std::io::Cursor::new(buf);
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| super::IoError::from_zip(e, path))?;

    for i in 0..archive.len() {
      let file = archive.by_index(i).map_err(|e| super::IoError::from_zip(e, path))?;
      f(file)?;
    }

    Ok(())
  }
}

async fn ensure_parent(path: &Path) -> IoResult<()> {
  if let Some(parent) = path.parent()
    && !parent.exists()
  {
    fs::create_dir_all(parent).await.map_err(|e| IoError::from_io(e, path))?;
  }
  Ok(())
}

// files

pub async fn create_file(path: impl AsRef<Path>) -> IoResult<fs::File> {
  let path = path.as_ref();
  let file = fs::File::create(path).await.map_err(|e| IoError::from_io(e, path))?;
  Ok(file)
}

pub async fn delete_file(path: impl AsRef<Path>) -> IoResult<()> {
  let path = path.as_ref();
  fs::remove_file(path).await.map_err(|e| IoError::from_io(e, path))?;
  Ok(())
}

pub async fn read(path: impl AsRef<Path>) -> IoResult<Vec<u8>> {
  let path = path.as_ref();
  let file = fs::File::open(path).await.map_err(|e| IoError::from_io(e, path))?;

  let mut rdr = BufReader::new(file);
  let mut content = Vec::new();

  rdr.read_to_end(&mut content).await.map_err(|e| IoError::from_io(e, path))?;

  Ok(content)
}

pub async fn write(path: impl AsRef<Path>, bytes: &[u8]) -> IoResult<()> {
  let path = path.as_ref();
  ensure_parent(path).await?;

  let tmp_path = path.with_extension("tmp");

  let result = async {
    let mut file = fs::File::create(&tmp_path).await?;
    file.write_all(bytes).await?;
    file.sync_all().await?;
    fs::rename(&tmp_path, path).await
  }
  .await;

  if let Err(e) = result {
    let _ = fs::remove_file(&tmp_path).await;
    return Err(IoError::from_io(e, path));
  }

  Ok(())
}

pub fn write_sync(path: impl AsRef<Path>, bytes: &[u8]) -> IoResult<()> {
  let path = path.as_ref();

  if let Some(parent) = path.parent()
    && !parent.exists()
  {
    std::fs::create_dir_all(parent).map_err(|e| IoError::from_io(e, path))?;
  }

  let temp_path = path.with_extension("tmp");

  let result = (|| {
    let mut file = std::fs::File::create(&temp_path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    std::fs::rename(&temp_path, path)
  })();

  if let Err(e) = result {
    let _ = std::fs::remove_file(&temp_path);
    return Err(IoError::from_io(e, path));
  }

  Ok(())
}

pub async fn is_old(path: impl AsRef<Path>, interval: &Duration) -> IoResult<bool> {
  let path = path.as_ref();
  let file = fs::File::open(path).await.map_err(|e| IoError::from_io(e, path))?;

  let metadata = file.metadata().await.map_err(|e| IoError::from_io(e, path))?;
  let modified = metadata.modified().map_err(|e| IoError::from_io(e, path))?;

  let age = SystemTime::now().duration_since(modified).unwrap_or_default();

  Ok(age > *interval)
}

pub async fn is_valid_sha1(path: impl AsRef<Path>, expected_sha1: &str) -> IoResult<bool> {
  let path = path.as_ref();

  let mut file = fs::File::open(path).await.map_err(|e| IoError::from_io(e, path))?;
  let mut hasher = sha1::Sha1::new();
  let mut buffer = [0; 8192];

  loop {
    let bytes_read = file.read(&mut buffer).await.map_err(|e| IoError::from_io(e, path))?;
    if bytes_read == 0 {
      break;
    }
    hasher.update(&buffer[..bytes_read]);
  }

  let digest = hasher.finalize();
  let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();

  Ok(hex == expected_sha1)
}

// directories

pub async fn create_dir_all(path: impl AsRef<Path>) -> IoResult<()> {
  let path = path.as_ref();
  fs::create_dir_all(path).await.map_err(|e| IoError::from_io(e, path))?;
  Ok(())
}

pub fn create_dir_all_sync(path: impl AsRef<Path>) -> IoResult<()> {
  let path = path.as_ref();
  std::fs::create_dir_all(path).map_err(|e| IoError::from_io(e, path))?;
  Ok(())
}

pub async fn get_subfolders(path: impl AsRef<Path>) -> IoResult<Vec<String>> {
  let path = path.as_ref();

  let metadata = fs::metadata(path).await.map_err(|e| IoError::from_io(e, path))?;
  if !metadata.is_dir() {
    return Ok(Vec::new());
  }

  let mut subdirs = Vec::new();
  let mut entries = fs::read_dir(path).await.map_err(|e| IoError::from_io(e, path))?;
  while let Some(entry) = entries.next_entry().await.map_err(|e| IoError::from_io(e, path))? {
    let entry_path = entry.path();

    if let Ok(metadata) = fs::metadata(&entry_path).await
      && metadata.is_dir()
      && let Some(name) = entry_path.file_name().and_then(|n| n.to_str())
    {
      subdirs.push(name.to_string());
    }
  }

  Ok(subdirs)
}
