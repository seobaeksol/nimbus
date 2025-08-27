//! Nimbus File Manager Library
//! 
//! Core library providing file management functionality
//! for the Nimbus cross-platform file manager.

// Re-export from main binary, not needed in lib

// Re-export core types
pub use core_engine::{FileInfo, FileSystem, FileError};

/// Application state shared across Tauri commands
#[derive(Default)]
pub struct AppState {
    // Add shared state here as the application grows
}

/// Common result type for Tauri commands
pub type CommandResult<T> = Result<T, String>;

/// Convert anyhow errors to string for Tauri
pub fn anyhow_to_string(err: anyhow::Error) -> String {
    err.to_string()
}

/// Application constants
pub mod constants {
    pub const APP_NAME: &str = "Nimbus";
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
}