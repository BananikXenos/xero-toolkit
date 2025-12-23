//! Common utilities for seasonal effects.

use adw::prelude::*;
use gtk4::{ApplicationWindow, DrawingArea, EventControllerMotion, Widget};
use std::cell::RefCell;
use std::rc::Rc;

/// Mouse position context for seasonal effects.
/// Provides mouse coordinates that effects can use.
pub struct MouseContext {
    position: Rc<RefCell<(f64, f64)>>,
}

impl MouseContext {
    /// Get a clone of the internal Rc<RefCell<(f64, f64)>> for sharing.
    pub fn position_internal(&self) -> Rc<RefCell<(f64, f64)>> {
        self.position.clone()
    }
}

/// Set up mouse tracking for the window and return a MouseContext.
pub fn setup_mouse_tracking(window: &ApplicationWindow) -> MouseContext {
    let mouse_pos = Rc::new(RefCell::new((0.0f64, 0.0f64)));

    let motion = EventControllerMotion::new();
    let mouse_pos_clone = mouse_pos.clone();
    motion.connect_motion(move |_, x, y| {
        *mouse_pos_clone.borrow_mut() = (x, y);
    });
    window.add_controller(motion);

    MouseContext {
        position: mouse_pos,
    }
}

/// Simple pseudo-random number generator for seasonal effects
pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        self.state
    }

    pub fn f64(&mut self) -> f64 {
        (self.next() % 1000000) as f64 / 1000000.0
    }
}

/// Helper function to add a drawing area as an overlay to the window.
pub fn add_overlay_to_window(window: &ApplicationWindow, drawing_area: &DrawingArea) -> bool {
    use log::info;

    let adw_window = match window.downcast_ref::<adw::ApplicationWindow>() {
        Some(w) => w,
        None => {
            info!("Window is not an AdwApplicationWindow, cannot add overlay");
            return false;
        }
    };

    let content_widget = match adw_window.content() {
        Some(w) => w,
        None => {
            info!("Window has no content");
            return false;
        }
    };

    // Verify it's a ToolbarView
    if content_widget.downcast_ref::<adw::ToolbarView>().is_none() {
        info!("Window content is not a ToolbarView, overlay may not work correctly");
    }

    // Check if the content is already wrapped in an overlay
    if let Some(existing_overlay) = content_widget.downcast_ref::<gtk4::Overlay>() {
        info!("Found existing overlay at window level, adding drawing area");
        existing_overlay.add_overlay(drawing_area);
        true
    } else {
        info!("Wrapping window content in overlay to cover entire window including navbar");
        // Create overlay that will wrap the entire content
        let overlay = gtk4::Overlay::new();
        
        // Remove the content from the window
        adw_window.set_content(Option::<&Widget>::None);
        
        // Add content as the main child of the overlay
        overlay.set_child(Some(&content_widget));
        
        // Add the drawing area as an overlay
        overlay.add_overlay(drawing_area);
        
        // Set the overlay as the window content
        adw_window.set_content(Some(&overlay));
        true
    }
}
