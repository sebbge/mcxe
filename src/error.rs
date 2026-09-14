#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("{0}")]
  Net(#[from] crate::net::NetError),

  #[error("{0}")]
  Io(#[from] crate::fs::IoError),

  #[error("Parse error: {0}")]
  Parse(#[from] serde_json::Error),

  #[error("Metadata error: {0}")]
  Metadata(String),

  #[error("Download error: {0}")]
  Download(String),
}
