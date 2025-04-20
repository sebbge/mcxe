#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("Build error: {0}")]
  Build(String),

  #[error("{0}")]
  Common(#[from] shared::Error),

  #[error("Parse error: {0}")]
  Parse(#[from] serde_json::Error),

  #[error("Metadata error: {0}")]
  Metadata(String),

  #[error("Launcher error: {0}")]
  Launcher(String),

  #[error("Download error: {0}")]
  Download(String),

  #[error("Join Error: {0}")]
  Join(#[from] tokio::task::JoinError),
}
