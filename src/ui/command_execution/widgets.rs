//! UI widgets for command execution dialog.

use gtk4::prelude::*;
use gtk4::{Button, Expander, Label, ProgressBar, TextBuffer, TextView, Window};
use std::rc::Rc;

/// Container for all command execution dialog widgets
pub struct CommandExecutionWidgets {
    pub window: Window,
    pub title_label: Label,
    pub progress_bar: ProgressBar,
    pub output_view: TextView,
    pub output_buffer: TextBuffer,
    pub cancel_button: Button,
    pub close_button: Button,
    pub expander: Expander,
}

/// Builder for CommandExecutionWidgets
pub struct WidgetsBuilder {
    window: Window,
    title_label: Label,
    progress_bar: ProgressBar,
    output_view: TextView,
    cancel_button: Button,
    close_button: Button,
    expander: Expander,
}

impl WidgetsBuilder {
    /// Create a new builder
    pub fn new(
        window: Window,
        title_label: Label,
        progress_bar: ProgressBar,
        output_view: TextView,
        cancel_button: Button,
        close_button: Button,
        expander: Expander,
    ) -> Self {
        Self {
            window,
            title_label,
            progress_bar,
            output_view,
            cancel_button,
            close_button,
            expander,
        }
    }

    /// Build the widgets container
    pub fn build(self) -> Rc<CommandExecutionWidgets> {
        let output_buffer = self.output_view.buffer();
        Rc::new(CommandExecutionWidgets {
            window: self.window,
            title_label: self.title_label,
            progress_bar: self.progress_bar,
            output_view: self.output_view,
            output_buffer,
            cancel_button: self.cancel_button,
            close_button: self.close_button,
            expander: self.expander,
        })
    }
}

impl CommandExecutionWidgets {
    /// Update progress bar with current step information
    pub fn update_progress(&self, current: usize, total: usize) {
        let progress = (current as f64) / (total as f64);
        self.progress_bar.set_fraction(progress);
        self.progress_bar
            .set_text(Some(&format!("Step {} of {}", current, total)));
    }

    /// Set the title label text
    pub fn set_title(&self, title: &str) {
        self.title_label.set_label(title);
    }

    /// Show completion state
    pub fn show_completion(&self, success: bool, message: &str) {
        self.set_title(message);
        self.cancel_button.set_visible(false);
        self.close_button.set_visible(true);
        self.close_button.set_sensitive(true);

        if success {
            self.progress_bar.set_fraction(1.0);
            self.progress_bar.set_text(Some("Completed"));
        } else {
            // Expand output on error
            self.expander.set_expanded(true);
        }
    }

    /// Disable the cancel button (when cancellation is in progress)
    pub fn disable_cancel(&self) {
        self.cancel_button.set_sensitive(false);
    }
}
