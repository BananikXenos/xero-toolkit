//! Shared dialog helpers used across the UI pages.

use gtk4::{AlertDialog, ApplicationWindow};

/// Show an error message dialog transient for the provided window.
pub fn show_error(window: &ApplicationWindow, message: &str) {
    let dialog = AlertDialog::builder()
        .message("Error")
        .detail(message)
        .modal(true)
        .build();

    dialog.show(Some(window));
}
