//! Tauri IPC commands
//!
//! Exposes Rust functionality to the frontend via Tauri's IPC system.

mod git;
mod parser;

pub use git::*;
pub use parser::*;
