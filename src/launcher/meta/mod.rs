mod assets;
pub use assets::AssetManager;
mod client;
pub use client::ClientManager;
mod library;
pub use library::LibraryManager;
mod version;
pub use version::{ManifestConfig, ManifestManager, VersionManager};

pub mod models;

mod process; // Metadata processing implementations
