//! Tauri IPC commands
//!
//! Exposes Rust functionality to the frontend via Tauri's IPC system.

mod git;
mod parser;
mod project;

pub use git::*;
pub use parser::*;
pub use project::*;
