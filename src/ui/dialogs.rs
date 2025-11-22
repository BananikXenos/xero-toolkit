//! Shared dialog helpers used across the UI pages.

use gtk4::prelude::*;
use gtk4::{ApplicationWindow, MessageDialog, MessageType, ButtonsType};

/// Show an error message dialog transient for the provided window.
pub fn show_error(window: &ApplicationWindow, message: &str) {
    let dialog = MessageDialog::builder()
        .transient_for(window)
        .modal(true)
        .message_type(MessageType::Error)
        .buttons(ButtonsType::Ok)
        .text("Error")
        .secondary_text(message)
        .build();

    dialog.connect_response(|dialog, _| dialog.close());
    dialog.present();
}

