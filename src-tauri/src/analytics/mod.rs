//! Analytics module for session and project statistics
//!
//! Story 2.34: Provides session-level metrics calculation and project-level
//! analytics aggregation for the statistics dashboard.
//!
//! ## Architecture
//!
//! - **Session Metrics**: Pre-computed during import, stored per-session
//! - **Project Analytics**: Aggregated on-demand from session metrics
//! - **Time Range Filtering**: Supports 7-day, 30-day, and all-time views
//!
//! ## Local First
//!
//! All calculations happen on the client side using local session data,
//! following the project's "Local First" architecture principle.

mod types;

#[cfg(test)]
mod types_tests;

pub use types::*;

/// Calculator module for metrics and analytics computation
pub mod calculator;

#[cfg(test)]
mod calculator_tests;
