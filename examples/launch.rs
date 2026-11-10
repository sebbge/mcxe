//! Launches a downloaded version in offline mode.
//!
//! Run `download` to get a game version first,
//! then `cargo run --example launch -- <version>`. Defaults to the latest release.

use mcxe::{LaunchConfig, Launcher};

#[tokio::main]
async fn main() -> mcxe::Result<()> {
  tracing_subscriber::fmt::init(); // logging

  let launcher = Launcher::default();
  launcher.init().await?;

  let version = match std::env::args().nth(1) {
    Some(version) => version,
    None => launcher.latest_versions().await.release,
  };

  let launch_conf = LaunchConfig::builder()
    .id("example_id")
    .version(&version)
    .username("Player12345")
    .directory("mcxe_example")
    .build();

  let process_handle = launcher.launch(launch_conf).await?;
  let state = process_handle.wait().await;

  println!(
    "{:?}: {} stdout lines, {} stderr lines",
    state.payload.status,
    state.payload.stdout.len(),
    state.payload.stderr.len()
  );

  Ok(())
}
