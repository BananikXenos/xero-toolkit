//! Drivers and hardware tools page button handlers.
//!
//! Handles:
//! - NVIDIA GPU drivers (closed and open source)
//! - Tailscale VPN
//! - ASUS ROG laptop tools

use crate::ui::dialogs::error::show_error;
use crate::ui::dialogs::selection::{
    show_selection_dialog, SelectionDialogConfig, SelectionOption,
};
use crate::ui::task_runner::{self, Command};
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button};
use log::{info, warn};

/// Set up all button handlers for the drivers page.
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder) {
    setup_gpu_drivers(page_builder);
    setup_tailscale(page_builder);
    setup_asus_rog(page_builder);
}

fn setup_gpu_drivers(builder: &Builder) {
    let Some(button) = builder.object::<Button>("btn_gpu_drivers") else {
        return;
    };

    button.connect_clicked(move |btn| {
        info!("GPU Drivers button clicked");

        let Some(window) = get_window(btn) else {
            return;
        };

        let window_clone = window.clone();

        let config = SelectionDialogConfig::new(
            "NVIDIA Driver Selection",
            "Select which NVIDIA driver version to install.",
        )
        .add_option(SelectionOption::new(
            "nvidia_closed",
            "NVIDIA Closed Source",
            "Proprietary NVIDIA drivers",
            false,
        ))
        .add_option(SelectionOption::new(
            "nvidia_open",
            "NVIDIA Open Source",
            "Open source NVIDIA drivers (Turing+ GPUs)",
            false,
        ))
        .add_option(SelectionOption::new(
            "cuda",
            "CUDA Toolkit",
            "NVIDIA CUDA Toolkit for GPU-accelerated computing",
            false,
        ))
        .confirm_label("Install");

        show_selection_dialog(window.upcast_ref(), config, move |selected| {
            // Check for driver conflict
            if selected.contains(&"nvidia_closed".to_string())
                && selected.contains(&"nvidia_open".to_string())
            {
                warn!("Both NVIDIA drivers selected - conflict");
                show_error(
                    &window_clone,
                    "Cannot install both closed and open source NVIDIA drivers.\nPlease select only one.",
                );
                return;
            }

            let commands = build_gpu_driver_commands(&selected);

            if !commands.is_empty() {
                task_runner::run(
                    window_clone.upcast_ref(),
                    commands,
                    "GPU Driver Installation",
                    None,
                );
            }
        });
    });
}

/// Build commands for selected GPU drivers.
fn build_gpu_driver_commands(selected: &[String]) -> Vec<Command> {
    let mut commands = Vec::new();

    if selected.contains(&"nvidia_closed".to_string()) {
        commands.push(Command::aur(
            &[
                "-S",
                "--needed",
                "--noconfirm",
                "libvdpau",
                "egl-wayland",
                "nvidia-dkms",
                "nvidia-utils",
                "opencl-nvidia",
                "libvdpau-va-gl",
                "nvidia-settings",
                "vulkan-icd-loader",
                "lib32-nvidia-utils",
                "lib32-opencl-nvidia",
                "linux-firmware-nvidia",
                "lib32-vulkan-icd-loader",
            ],
            "Installing NVIDIA proprietary drivers...",
        ));
    }

    if selected.contains(&"nvidia_open".to_string()) {
        commands.push(Command::aur(
            &[
                "-S",
                "--needed",
                "--noconfirm",
                "libvdpau",
                "egl-wayland",
                "nvidia-utils",
                "opencl-nvidia",
                "libvdpau-va-gl",
                "nvidia-settings",
                "nvidia-open-dkms",
                "vulkan-icd-loader",
                "lib32-nvidia-utils",
                "lib32-opencl-nvidia",
                "linux-firmware-nvidia",
                "lib32-vulkan-icd-loader",
            ],
            "Installing NVIDIA open source drivers...",
        ));
    }

    if selected.contains(&"cuda".to_string()) {
        commands.push(Command::aur(
            &["-S", "--needed", "--noconfirm", "cuda", "cudnn"],
            "Installing CUDA Toolkit...",
        ));
    }

    // Run NVIDIA post-install configuration if a driver was selected
    let driver_selected = selected.contains(&"nvidia_closed".to_string())
        || selected.contains(&"nvidia_open".to_string());

    if driver_selected {
        commands.push(Command::privileged(
            "bash",
            &["/opt/xero-toolkit/scripts/nv-setup.sh"],
            "Configuring NVIDIA drivers...",
        ));
    }

    commands
}

fn setup_tailscale(builder: &Builder) {
    let Some(button) = builder.object::<Button>("btn_tailscale") else {
        return;
    };

    button.connect_clicked(move |btn| {
        info!("Tailscale VPN button clicked");

        let Some(window) = get_window(btn) else {
            return;
        };

        let commands = vec![Command::privileged(
            "bash",
            &[
                "-c",
                "curl -fsSL https://raw.githubusercontent.com/xerolinux/xero-fixes/main/conf/install.sh | bash",
            ],
            "Installing Tailscale VPN...",
        )];

        task_runner::run(window.upcast_ref(), commands, "Install Tailscale VPN", None);
    });
}

fn setup_asus_rog(builder: &Builder) {
    let Some(button) = builder.object::<Button>("btn_asus_rog") else {
        return;
    };

    button.connect_clicked(move |btn| {
        info!("ASUS ROG Tools button clicked");

        let Some(window) = get_window(btn) else {
            return;
        };

        let commands = vec![
            Command::aur(
                &[
                    "-S",
                    "--noconfirm",
                    "--needed",
                    "rog-control-center",
                    "asusctl",
                    "supergfxctl",
                ],
                "Installing ASUS ROG control tools...",
            ),
            Command::privileged(
                "systemctl",
                &["enable", "--now", "asusd", "supergfxd"],
                "Enabling ASUS ROG services...",
            ),
        ];

        task_runner::run(
            window.upcast_ref(),
            commands,
            "Install ASUS ROG Tools",
            None,
        );
    });
}

/// Helper to get the parent window from a button.
fn get_window(button: &Button) -> Option<ApplicationWindow> {
    button
        .root()
        .and_then(|root| root.downcast::<ApplicationWindow>().ok())
}
