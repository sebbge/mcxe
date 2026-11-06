[![Crates.io](https://img.shields.io/crates/v/comet-minecraft)](https://crates.io/crates/comet-minecraft)
[![Docs.rs](https://img.shields.io/docsrs/comet-minecraft)](https://docs.rs/comet-minecraft)
[![CI](https://github.com/bllexz/comet/actions/workflows/ci.yml/badge.svg)](https://github.com/bllexz/comet/actions/workflows/ci.yml)
![License](https://img.shields.io/crates/l/comet-minecraft)

# comet

A Rust crate that lets you download and launch Minecraft: Java Edition effortlessly.

```toml
[dependencies]
comet-minecraft = "0.1"
```

## Example

```rust
use comet::{LaunchConfig, Launcher, LauncherConfig};

#[tokio::main]
async fn main() -> comet::Result<()> {
  let launcher = Launcher::new(LauncherConfig::builder().root_dir("<launcher_directory>").build());
  launcher.init().await?;

  let version = launcher.latest_versions().await.release;

  let (java, game) = launcher.download(&version).await?;
  if let Some(java) = java {
    java.wait().await;
  }
  game.on_update(|state| println!("[{:>3}%] {}", state.payload.progress, state.payload.message));
  game.wait().await;

  let config = LaunchConfig::builder()
    .id("main")
    .version(&version)
    .username("Player")
    .directory("main")
    .build();

  let state = launcher.launch(config).await?.wait().await;
  println!("{:?}", state.payload.status);
  Ok(())
}
```

More in [`examples/`](examples).

## Loaders

| Loader          | Support     | Notes                              |
| --------------- | ----------- | ---------------------------------- |
| Vanilla         | Yes         | Full support                       |
| Fabric          | Launch only | Installing the loader is up to you |
| Quilt           | Launch only | Installing the loader is up to you |
| Forge, NeoForge | No          | Processors not supported           |

Currently using Azul Zulu JRE distributions.

## Feature flags

- `ts-types`: derives [ts-rs](https://github.com/Aleph-Alpha/ts-rs) on the status and version types.
  `cargo test --features ts-types` writes them to `bindings/`.

## Roadmap

- Custom mirror support for different JREs or third-party clients
- Forge and NeoForge support (installer processors)
- Microsoft authentication (optional feature)
- CLI (optional feature)
- Automatic mod loader installation
- Server download and launch

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.

Not affiliated with Mojang or Microsoft.
