//! Tauri command handlers
//! 
//! This module contains all the command handlers that provide
//! the bridge between the React frontend and Rust backend.

pub mod files;
pub mod search;
pub mod system;
#[cfg(test)]
mod tests;

// Common types for command results
pub type CommandResult<T> = Result<T, String>;

// Utility function to convert anyhow errors to strings
#[allow(dead_code)]
pub fn anyhow_to_string(err: anyhow::Error) -> String {
    format!("{}", err)
}