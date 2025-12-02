//! Tab navigation and sidebar management.
//!
//! This module handles the sidebar navigation tabs that allow users
//! to switch between different pages in the application.

use crate::ui::context::UiComponents;
use gtk4::prelude::*;
use gtk4::{Box, Button, Image, Label, Orientation, Stack};
use log::info;

/// Tab configuration: (label, page_name, icon_name)
const TABS: &[(&str, &str, &str)] = &[
    ("Main Page", "main_page", "house-symbolic"),
    ("Drivers", "drivers", "gear-symbolic"),
    ("Customization", "customization", "brush-symbolic"),
    ("Gaming Tools", "gaming_tools", "gamepad-symbolic"),
    ("Containers/VMs", "containers_vms", "box-symbolic"),
    ("Multimedia Tools", "multimedia_tools", "play-symbolic"),
    (
        "Kernel Manager/SCX",
        "kernel_manager_scx",
        "hammer-symbolic",
    ),
    (
        "Servicing/System tweaks",
        "servicing_system_tweaks",
        "toolbox-symbolic",
    ),
];

/// Represents a single tab in the navigation sidebar.
struct Tab {
    page_name: String,
    button: Button,
}

impl Tab {
    /// Create a new tab with the given label, page name and icon.
    fn new(label: &str, page_name: &str, icon_name: &str) -> Self {
        let content_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .hexpand(true)
            .build();

        let image = Image::from_icon_name(icon_name);
        image.set_pixel_size(18);

        let label_widget = Label::new(Some(label));
        label_widget.set_xalign(0.0);

        content_box.append(&image);
        content_box.append(&label_widget);

        let button = Button::builder()
            .hexpand(true)
            .css_classes(vec!["tab-button".to_string()])
            .build();

        button.set_child(Some(&content_box));

        Tab {
            page_name: page_name.to_string(),
            button,
        }
    }

    /// Connect this tab's button to navigate to its page.
    fn connect(&self, stack: &Stack, tabs_container: &Box) {
        let stack_clone = stack.clone();
        let page_name = self.page_name.clone();
        let button_clone = self.button.clone();
        let tabs_clone = tabs_container.clone();

        self.button.connect_clicked(move |_| {
            info!("Navigating to page '{}'", page_name);
            stack_clone.set_visible_child_name(&page_name);
            update_active_tab(&tabs_clone, &button_clone);
        });
    }
}

/// Set up the navigation tabs in the sidebar.
pub fn setup(ui: &UiComponents) {
    info!("Setting up navigation tabs");

    let tabs_container = ui.tabs_container();
    let stack = ui.stack();

    let mut first_button: Option<Button> = None;

    for &(label, page_name, icon_name) in TABS {
        let tab = Tab::new(label, page_name, icon_name);
        tab.connect(stack, tabs_container);

        if first_button.is_none() {
            first_button = Some(tab.button.clone());
        }

        tabs_container.append(&tab.button);
        info!("Added tab: {} -> '{}'", label, page_name);
    }

    // Set first tab as active
    if let Some(button) = first_button {
        button.add_css_class("active");
    }
}

/// Update which tab is marked as active.
fn update_active_tab(tabs_container: &Box, clicked_button: &Button) {
    let mut child = tabs_container.first_child();

    while let Some(widget) = child {
        if let Ok(button) = widget.clone().downcast::<Button>() {
            if button == *clicked_button {
                button.add_css_class("active");
            } else {
                button.remove_css_class("active");
            }
        }
        child = widget.next_sibling();
    }
}
