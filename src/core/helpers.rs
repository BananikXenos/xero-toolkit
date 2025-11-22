//! Helper functions for package and system checks.

use crate::utils;
use log::debug;

/// Check if a package is installed using AUR helper (or fallback to pacman)
pub fn is_package_installed(package: &str) -> bool {
    debug!("Checking if package '{}' is installed", package);

    if let Some(helper) = utils::detect_aur_helper() {
        if let Ok(output) = std::process::Command::new(helper)
            .args(["-Q", package])
            .output()
        {
            if output.status.success() {
                debug!("Package '{}' found via {}", package, helper);
                return true;
            }
        }
    }

    let installed = std::process::Command::new("pacman")
        .args(["-Q", package])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if installed {
        debug!("Package '{}' found via pacman", package);
    } else {
        debug!("Package '{}' not installed", package);
    }

    installed
}

/// Check if a flatpak package is installed (apps, runtimes, and extensions)
pub fn is_flatpak_installed(package: &str) -> bool {
    debug!("Checking if Flatpak '{}' is installed", package);

    let installed = std::process::Command::new("flatpak")
        .args(["list"])
        .output()
        .map(|output| {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).contains(package)
            } else {
                false
            }
        })
        .unwrap_or(false);

    if installed {
        debug!("Flatpak '{}' found", package);
    } else {
        debug!("Flatpak '{}' not installed", package);
    }

    installed
}

/// Check if a systemd service is enabled
#[allow(dead_code)]
pub fn is_service_enabled(service: &str) -> bool {
    debug!("Checking if service '{}' is enabled", service);

    std::process::Command::new("systemctl")
        .args(["is-enabled", service])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Check if a systemd service is active (running)
#[allow(dead_code)]
pub fn is_service_active(service: &str) -> bool {
    debug!("Checking if service '{}' is active", service);

    std::process::Command::new("systemctl")
        .args(["is-active", service])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Check if a command is available in PATH
#[allow(dead_code)]
pub fn is_command_available(command: &str) -> bool {
    debug!("Checking if command '{}' is available", command);

    std::process::Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Get the list of installed packages matching a pattern
#[allow(dead_code)]
pub fn list_packages_matching(pattern: &str) -> Vec<String> {
    debug!("Listing packages matching pattern '{}'", pattern);

    if let Some(helper) = utils::detect_aur_helper() {
        if let Ok(output) = std::process::Command::new(helper).args(["-Q"]).output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                return stdout
                    .lines()
                    .filter(|line| line.contains(pattern))
                    .map(|line| line.split_whitespace().next().unwrap_or("").to_string())
                    .collect();
            }
        }
    }

    if let Ok(output) = std::process::Command::new("pacman").args(["-Q"]).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout
                .lines()
                .filter(|line| line.contains(pattern))
                .map(|line| line.split_whitespace().next().unwrap_or("").to_string())
                .collect();
        }
    }

    Vec::new()
}

/// Check if multiple packages are installed
#[allow(dead_code)]
pub fn are_packages_installed(packages: &[&str]) -> Vec<(String, bool)> {
    packages
        .iter()
        .map(|pkg| (pkg.to_string(), is_package_installed(pkg)))
        .collect()
}

/// Check if multiple flatpaks are installed
#[allow(dead_code)]
pub fn are_flatpaks_installed(packages: &[&str]) -> Vec<(String, bool)> {
    packages
        .iter()
        .map(|pkg| (pkg.to_string(), is_flatpak_installed(pkg)))
        .collect()
}

/// Check if a file exists at the given path
#[allow(dead_code)]
pub fn file_exists(path: &str) -> bool {
    std::path::Path::new(path).exists()
}

/// Check if a directory exists at the given path
#[allow(dead_code)]
pub fn directory_exists(path: &str) -> bool {
    std::path::Path::new(path).is_dir()
}

/// Get system distribution information
#[allow(dead_code)]
pub fn get_distribution_info() -> Option<(String, String)> {
    if let Ok(contents) = std::fs::read_to_string("/etc/os-release") {
        let mut name = None;
        let mut version = None;

        for line in contents.lines() {
            if line.starts_with("NAME=") {
                name = Some(
                    line.trim_start_matches("NAME=")
                        .trim_matches('"')
                        .to_string(),
                );
            } else if line.starts_with("VERSION=") {
                version = Some(
                    line.trim_start_matches("VERSION=")
                        .trim_matches('"')
                        .to_string(),
                );
            }
        }

        if let (Some(n), Some(v)) = (name, version) {
            return Some((n, v));
        }
    }

    None
}

/// Check if running on XeroLinux
#[allow(dead_code)]
pub fn is_xerolinux() -> bool {
    if let Ok(contents) = std::fs::read_to_string("/etc/os-release") {
        return contents.contains("XeroLinux") || contents.contains("xerolinux");
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_exists() {
        assert!(file_exists("/etc/os-release"));
        assert!(!file_exists("/nonexistent/file"));
    }

    #[test]
    fn test_directory_exists() {
        assert!(directory_exists("/etc"));
        assert!(!directory_exists("/nonexistent/directory"));
    }
}
