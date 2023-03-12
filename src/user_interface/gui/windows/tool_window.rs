use egui::{Context, Align2};

use crate::{editor::Editor, user_interface::{Interface, gui::window::GlifWindow}};


pub struct ToolWindow {
    // is this window open?
    open: bool,
}

impl ToolWindow {
    pub fn new() -> Self {
        Self { open: true }
    }
}

impl GlifWindow for ToolWindow {
    fn open(&self) -> bool {
        self.open
    }

    fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    fn build(&mut self, ctx: &Context, v: &mut Editor, i: &mut Interface) {
        // prepare for a hack
        let mut populated_ui = false;
        egui::Window::new("Dummy")
            .fixed_pos([1000000000., 10000000000.])
            .show(ctx, |ui| {
                populated_ui = v.dispatch_tool_ui(i, ui);
            });

        if !populated_ui { return };

        egui::Window::new("Tool")
            .resizable(false)
            .collapsible(true)
            .enabled(!v.is_modifying())
            .default_width(150.)
            .anchor(Align2::RIGHT_TOP, egui::vec2(0., 0.))
            .constrain(true)
            .show(ctx, |ui| {
                v.dispatch_tool_ui(i, ui);
            });
    }
}