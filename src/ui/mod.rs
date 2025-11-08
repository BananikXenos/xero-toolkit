//! User Interface handling functionality.
//!
//! This module contains all UI-related components organized by functionality:
//! - `app`: Application setup and initialization
//! - `tabs`: Tab navigation and management

pub mod app;
pub mod tabs;

// Re-export commonly used items
pub use app::setup_application_ui;
