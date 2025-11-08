//! Centralized configuration and constants for the application.

/// Application information constants.
pub mod app_info {
    pub const NAME: &str = "XFPrintD GUI";
    pub const ID: &str = "xyz.xerolinux.xfprintd_gui";
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
}
