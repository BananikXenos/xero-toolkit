//! Utility functions for system operations and path management.

use log::debug;
use std::env;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// Detect available AUR helper in priority order.
///
/// Searches for AUR helpers in the following order:
/// 1. paru
/// 2. yay
///
/// Returns the first found helper or None if none are available.
pub fn detect_aur_helper() -> Option<&'static str> {
    const PRIORITY: [&str; 2] = ["paru", "yay"];

    for &cmd in PRIORITY.iter() {
        if is_executable_in_path(cmd) {
            debug!("Found AUR helper: {}", cmd);
            return Some(cmd);
        }
    }

    debug!("No AUR helper found");
    None
}

/// Check if a command is executable in PATH.
///
/// If the command contains a path separator, checks if the file exists directly.
/// Otherwise, searches through PATH directories for the executable.
fn is_executable_in_path(cmd: &str) -> bool {
    if cmd.contains(std::path::MAIN_SEPARATOR) {
        return PathBuf::from(cmd).is_file();
    }

    let paths = match env::var_os("PATH") {
        Some(p) => p,
        None => return false,
    };

    for dir in env::split_paths(&paths) {
        let mut candidate = dir.clone();
        candidate.push(cmd);
        if candidate.exists() {
            if let Ok(metadata) = std::fs::metadata(&candidate) {
                let perms = metadata.permissions();
                if perms.mode() & 0o111 != 0 {
                    return true;
                }
            }
        }
    }

    false
}

/// Open a URL in the default browser.
///
/// Uses xdg-open on Linux systems.
pub fn open_url(url: &str) -> Result<(), std::io::Error> {
    debug!("Opening URL: {}", url);
    std::process::Command::new("xdg-open").arg(url).spawn()?;
    Ok(())
}
