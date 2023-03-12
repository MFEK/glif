use std::collections::HashMap;

use super::egui_parsed_textfield;
use crate::{
    editor::Editor,
    user_interface::{gui::window::GlifWindow, Interface},
};
use egui::Context;

pub struct GridWindow {
    // is this window open?
    open: bool,
    edit_buf: HashMap<String, String>,
}

impl GridWindow {
    pub fn new() -> Self {
        Self {
            open: false,
            edit_buf: HashMap::new(),
        }
    }
}

impl GlifWindow for GridWindow {
    fn open(&self) -> bool {
        self.open
    }

    fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    fn build(&mut self, ctx: &Context, v: &mut Editor, i: &mut Interface) {
        egui::Window::new("Grid")
            .resizable(true)
            .collapsible(true)
            .open(&mut self.open)
            .enabled(!v.is_modifying())
            .constrain(true)
            .default_width(100.)
            .show(ctx, |ui| {
                ui.checkbox(&mut i.grid.show, "Active");

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Spacing");
                    i.grid.spacing =
                        egui_parsed_textfield(ui, "spacing", i.grid.spacing, &mut self.edit_buf);
                });

                ui.horizontal(|ui| {
                    ui.label("Offset");
                    i.grid.offset =
                        egui_parsed_textfield(ui, "offset", i.grid.spacing, &mut self.edit_buf);
                });

                let prev_italic = i.grid.slope.is_some();
                let mut italic = i.grid.slope.is_some();
                ui.checkbox(&mut italic, "Italic");

                if italic != prev_italic && italic {
                    i.grid.slope = Some(0.5);
                } else if italic != prev_italic && !italic {
                    i.grid.slope = None;
                }

                if let Some(slope) = i.grid.slope {
                    i.grid.slope = Some(egui_parsed_textfield(
                        ui,
                        "slope",
                        slope,
                        &mut self.edit_buf,
                    ));

                    let mut angle = (f32::to_degrees(f32::atan(slope)) * 10000.).round() / 10000.;
                    angle = egui_parsed_textfield(ui, "spacing", angle, &mut self.edit_buf);
                }

                i.grid.offset %= i.grid.spacing;
            });
    }
}
