//! Downloads a version and the Java runtime it needs, reporting progress.
//!
//! Run with `cargo run --example download -- <version>`. Defaults to the latest release.

use mcxe::Launcher;

#[tokio::main]
async fn main() -> mcxe::Result<()> {
  tracing_subscriber::fmt::init(); // logging

  let launcher = Launcher::default();
  launcher.init().await?;

  let version = match std::env::args().nth(1) {
    Some(version) => version,
    None => launcher.latest_versions().await.release,
  };

  let (java, game) = launcher.download(&version).await?;

  if let Some(java) = java {
    java.on_update(|state| println!("java [{:>3}%] {}", state.payload.progress, state.payload.message));
  }

  game.on_update(|state| println!("game [{:>3}%] {}", state.payload.progress, state.payload.message));

  let state = game.wait().await;
  println!(
    "{:?} in {:.1?} {}",
    state.payload.status,
    state.duration().unwrap_or_default(),
    state.payload.error_message
  );

  Ok(())
}
