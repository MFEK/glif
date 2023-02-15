use super::super::prelude::*;
use super::PAP;
use crate::user_interface::{self, Interface, gui::windows::egui_parsed_textfield};
use glifparser::glif::contour_operations::{pap::{PatternStretch, PatternCopies, PatternSubdivide, PAPContour}, ContourOperations};
use egui::Ui;

impl PAP {
    pub fn tool_dialog(&mut self, v: &mut Editor, i: &Interface, ui: &mut Ui) {
        if v.contour_idx.is_none() { return };
        let contour_idx = v.contour_idx.expect("Checked in function return gaurd.");

        let operation = v.get_active_layer_ref().outline[contour_idx]
            .operation()
            .clone();


        if let Some(ContourOperations::PatternAlongPath { mut data }) = operation {
            let original_data = data.clone();

            egui::ComboBox::from_label("Mode")
                .selected_text(format!("{:?}", data.copies))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut data.copies, PatternCopies::Single, "Single");
                    ui.selectable_value(&mut data.copies, PatternCopies::Repeated, "Repeated");
                });

            let mut subdivisions = match data.subdivide {
                PatternSubdivide::Simple(times) => times,
                _ => 0,
            } as f32;
            
            let subdivision_slider = egui::Slider::new(&mut subdivisions, 0.0..=3.0)
                .text("Subdivisions")
                .step_by(1.0);

            ui.add(subdivision_slider);

            data.subdivide = match subdivisions {
                0. => {
                    PatternSubdivide::Off
                }
                _ => {
                    PatternSubdivide::Simple(subdivisions as usize)
                }
            };

            ui.checkbox(&mut data.center_pattern, "Center");

            egui::ComboBox::from_label("Stretch")
            .selected_text(format!("{:?}", data.stretch))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut data.stretch, PatternStretch::Off, "Off");
                ui.selectable_value(&mut data.stretch, PatternStretch::On, "On");
                ui.selectable_value(&mut data.stretch, PatternStretch::Spacing, "Spacing");
            });

            ui.checkbox(&mut data.simplify, "Simplify");

            ui.horizontal(|ui| {
                ui.label("Spacing:");
                data.spacing = egui_parsed_textfield(ui, "Spacing", data.spacing as f32, &mut self.edit_buf) as f64;
            });

            ui.add(egui::Slider::new(&mut data.normal_offset, -100.0..=100.0)
                .text("Normal Offset")
            );

            ui.add(egui::Slider::new(&mut data.tangent_offset, -100.0..=100.0)
                .text("Tangent Offset")
            );

            ui.add(egui::Slider::new(&mut data.pattern_scale.0, -10.0..=10.0)
                .text("Scale X")
            );

            ui.add(egui::Slider::new(&mut data.pattern_scale.1, -10.0..=10.0)
                .text("Scale Y")
            );

            ui.add(egui::Slider::new(&mut data.prevent_overdraw, 0.0..=1.0)
                .text("Prevent Overdraw")
            );

            if data != original_data {
                v.begin_modification("Modified PAP.", true);
                *v.get_active_layer_mut().outline[contour_idx].operation_mut() = Some(ContourOperations::PatternAlongPath { data: data });
                v.end_modification();
            }
        } else if v.selected_point().is_some() {
            let none_string = "None".to_string();
            let selected_string = if let Some(layer) = self.pattern_layer {
                if let Some(l) = v.with_glyph(|glif| glif.layers.get(layer).cloned()) {
                    l.name.clone()
                } else if layer == v.get_active_layer() {
                    self.pattern_layer = None;
                    none_string
                } else {
                    self.pattern_layer = None;
                    none_string
                }
            } else { none_string };

            egui::ComboBox::new("pattern", "Pattern Layer")
                .selected_text(selected_string.as_str())
                .show_ui(ui, |ui| {
                    for i in 1..v.get_layer_count() {
                        let layer_name = v.with_glyph(|glif| glif.layers[i].name.clone());
                        ui.selectable_value(&mut self.pattern_layer, Some(i), layer_name.as_str());
                    }
                });


            let contour_idx = v.contour_idx.expect("Checked in if statement above.");
            let enabled = self.pattern_layer.is_some();
            if ui.add_enabled(enabled, egui::Button::new("Create Pattern Along Path.")).clicked() {
                let pattern = v.with_glyph(|glif| glif.layers[self.pattern_layer.unwrap()].clone()).outline;
                v.begin_modification("Added PAP contour.", true);
                v.get_active_layer_mut().outline[contour_idx].set_operation(
                    Some(ContourOperations::PatternAlongPath {
                        // TODO: Default() implementation for many of our structs.
                        data: PAPContour {
                            pattern: pattern,
                            copies: PatternCopies::Repeated,
                            subdivide: PatternSubdivide::Off,
                            is_vertical: false,
                            stretch: PatternStretch::On,
                            spacing: 4.,
                            simplify: false,
                            normal_offset: 0.,
                            tangent_offset: 0.,
                            pattern_scale: (1., 1.),
                            center_pattern: true,
                            prevent_overdraw: 0.,
                            two_pass_culling: false,
                            reverse_path: false,
                            reverse_culling: false,
                        },
                    })
                );
                v.end_modification();
            }
        } else {
            ui.label("No selection!");
        }
    }
}
