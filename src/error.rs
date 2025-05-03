#[derive(thiserror::Error, Debug)]
pub enum Error {
  // shared
  #[error("{0}")]
  Net(#[from] crate::net::NetError),

  #[error("{0}")]
  Io(#[from] crate::io::IoError),

  // launcher (this)
  #[error("Parse error: {0}")]
  Parse(#[from] serde_json::Error),

  #[error("Metadata error: {0}")]
  Metadata(String),

  #[error("Launcher error: {0}")]
  Launcher(String),

  #[error("Download error: {0}")]
  Download(String),
}

impl From<shared::Error> for Error {
  fn from(e: shared::Error) -> Self {
    match e {
      shared::Error::Net(e) => Self::Net(e),
      shared::Error::Io(e) => Self::Io(e),
    }
  }
}
