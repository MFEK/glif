use super::super::prelude::*;
use super::Dash;
use crate::user_interface::{self, Interface};
use glifparser::glif::contour_operations::{ContourOperations, dash::DashCull};
use egui::Ui;
use skia_safe::{PaintCap, PaintJoin};

impl Dash {
    pub fn tool_dialog(&mut self, v: &mut Editor, i: &Interface, ui: &mut Ui) {
        if v.contour_idx.is_none() {
            ui.label("No selection!");
            return;
        }

        let contour_idx = v.contour_idx.unwrap();
        let mut operation = v.get_active_layer_ref().outline[contour_idx]
            .operation()
            .clone();

        if let Some(ContourOperations::DashAlongPath { mut data }) = operation {
            let original_data = data.clone();

            ui.label("Stroke Width");
            let width_slider = egui::Slider::new(&mut data.stroke_width, 0.0..=30.);
            ui.add(width_slider);
        
            let mut dashes = data.dash_desc
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(" ");

            ui.label("Dashes");
            if ui.text_edit_singleline(&mut dashes).changed() && !dashes.as_str().is_empty() {
                let new = dashes
                    .as_str()
                    .split(" ")
                    .map(|s| s.parse::<f32>())
                    .filter(|f| f.is_ok())
                    .map(|o| o.unwrap())
                    .collect::<Vec<_>>();

                if new.len() > 1 && new.len() % 2 == 0 {
                    data.dash_desc = new;
                }
            }
            
            let mut cull = data.cull.is_some();
            ui.checkbox(&mut cull, "Cull");

            if cull {
                ui.checkbox( &mut data.include_last_path, "Include last?");

                let mut cull_width = data.cull.map(|c| c.width as f32).unwrap_or(0.);
                let cull_width_slider = egui::Slider::new(&mut cull_width, 0.0..=30.);

                ui.add(cull_width_slider);

                let mut cull_area_cutoff = data.cull.map(|c| c.area_cutoff as f32).unwrap_or(0.);
                let cull_area_cutoff_slider = egui::Slider::new(&mut cull_area_cutoff, 0.0..=30.);

                ui.add(cull_area_cutoff_slider);

                const JOINS: [u8; 3] = [
                    PaintJoin::Miter as u8,
                    PaintJoin::Bevel as u8,
                    PaintJoin::Round as u8,
                ];

                let mut join = JOINS.iter().position(|&r| r == data.paint_join).unwrap();
                egui::ComboBox::new("Joins", "Joins")
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut join, 0, "Miter");
                        ui.selectable_value(&mut join, 1, "Bevel");
                        ui.selectable_value(&mut join, 2, "Round");
                    });

                data.paint_join = JOINS[join];

                const CAPS: [u8; 3] = [
                    PaintCap::Butt as u8,
                    PaintCap::Square as u8,
                    PaintCap::Round as u8,
                ];

                let mut cap = CAPS.iter().position(|&r| r == data.paint_cap).unwrap();
                egui::ComboBox::new("Caps", "Caps")
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut cap, 0, "Butt");
                        ui.selectable_value(&mut cap, 1, "Square");
                        ui.selectable_value(&mut cap, 2, "Round");
                    });

                data.paint_cap = CAPS[cap];
                
                data.cull = Some(DashCull {
                        width: cull_width,
                        area_cutoff: cull_area_cutoff,
                    });
            } else {
                data.cull = None;
            }

            if data != original_data {
                v.begin_modification("Dash dialog modification.", true);
                v.get_active_layer_mut().outline[contour_idx].set_operation(Some(ContourOperations::DashAlongPath { data }));
                v.end_modification();
            }
        }
    }
}
