use crate::{fs::IoError, net::NetError};

use futures::StreamExt;
use std::path::Path;
use tokio::io::AsyncWriteExt;

pub async fn stream_to_file<E>(
  response: reqwest::Response,
  path: impl AsRef<Path>,
  cancel: impl Fn() -> bool,
  on_progress: impl Fn(usize),
) -> Result<Option<()>, E>
where
  E: From<IoError> + From<NetError>,
{
  let path = path.as_ref();
  let mut stream = response.bytes_stream();
  let mut file = crate::fs::create_file(path).await?;

  while let Some(chunk_result) = stream.next().await {
    if cancel() {
      return Ok(None);
    }

    let chunk = chunk_result?;
    file.write_all(&chunk).await.map_err(|e| IoError::from_io(e, path))?;
    on_progress(chunk.len());
  }

  file.sync_all().await.map_err(|e| IoError::from_io(e, path))?;
  Ok(Some(()))
}
