use reqwest::{Client, Response};
use std::sync::OnceLock;

pub type NetError = reqwest::Error;
pub type NetResult<T> = std::result::Result<T, NetError>;

#[derive(thiserror::Error, Debug)]
#[error("HTTP client already initialized")]
pub struct ClientAlreadyInitialized;

static CLIENT: OnceLock<Client> = OnceLock::new();

pub fn set_client(client: Client) -> Result<(), ClientAlreadyInitialized> {
  CLIENT.set(client).map_err(|_| ClientAlreadyInitialized)
}

pub fn init_client(user_agent: impl Into<String>, timeout_secs: u64) -> Result<(), ClientAlreadyInitialized> {
  let client = Client::builder()
    .timeout(std::time::Duration::from_secs(timeout_secs))
    .user_agent(user_agent.into())
    .build()
    .expect("Failed to build HTTP client");

  set_client(client)
}
//

pub(crate) fn client() -> &'static Client {
  CLIENT.get_or_init(|| {
    Client::builder()
      .timeout(std::time::Duration::from_secs(30))
      .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
      .build()
      .expect("Failed to build HTTP client")
  })
}

pub(crate) async fn get(url: &str) -> NetResult<Response> {
  client().get(url).send().await?.error_for_status()
}

pub(crate) async fn get_bytes(url: &str) -> NetResult<Vec<u8>> {
  let bytes = get(url).await?.bytes().await?;
  Ok(bytes.into())
}
