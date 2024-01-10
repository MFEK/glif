use super::super::prelude::*;
use super::PAP;
use crate::user_interface::Interface;
use glifparser::glif::contour_operations::{pap::{PatternStretch, PatternCopies, PatternSubdivide, PAPContour, PatternCulling}, ContourOperations};
use egui::Ui;

#[derive(PartialEq, Clone, Copy, Debug)]
enum PAPSubdivide {
    Off,
    Simple,
    Angle,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum PAPCulling {
    Off,
    RemoveOverlapping,
    EraseOverlapping,
}

impl PAP {
    pub fn tool_dialog(&mut self, v: &mut Editor, _i: &Interface, ui: &mut Ui) {
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

            egui::CollapsingHeader::new("Subdivision").show(ui, |ui| {
                let prev_subdivide_mode = match data.subdivide {
                    PatternSubdivide::Off => PAPSubdivide::Off,
                    PatternSubdivide::Simple(_) => PAPSubdivide::Simple,
                    PatternSubdivide::Angle(_) => PAPSubdivide::Angle,
                };

                let mut subdivide_mode = prev_subdivide_mode.clone();
                egui::ComboBox::from_label("Mode")
                    .selected_text(format!("{:?}", subdivide_mode))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut subdivide_mode, PAPSubdivide::Off, "Off");
                        ui.selectable_value(&mut subdivide_mode, PAPSubdivide::Simple, "Simple");
                        ui.selectable_value(&mut subdivide_mode, PAPSubdivide::Angle, "Angle");
                    });

                if subdivide_mode != prev_subdivide_mode {
                    match subdivide_mode {
                        PAPSubdivide::Off => data.subdivide = PatternSubdivide::Off,
                        PAPSubdivide::Simple => data.subdivide = PatternSubdivide::Simple(2),
                        PAPSubdivide::Angle => data.subdivide = PatternSubdivide::Angle(5.0),
                    }
                }

                data.subdivide = match data.subdivide {
                    PatternSubdivide::Simple(times) => {
                        let mut times = times as i32;
                        ui.add(egui::Slider::new(&mut times, 1..=10)
                            .text("Times")
                        );
                        PatternSubdivide::Simple(times as usize)
                    }

                    PatternSubdivide::Angle(angle) => {
                        let mut angle = angle as f32;
                        ui.add(egui::Slider::new(&mut angle, 0.0..=90.0)
                            .text("Angle")
                        );
                        PatternSubdivide::Angle(angle as f64)
                    }

                    _ => { PatternSubdivide::Off }
                }
            });

            egui::CollapsingHeader::new("Culling").show(ui, |ui| {
                let prev_culling_mode: PAPCulling = match data.prevent_overdraw {
                    PatternCulling::Off => PAPCulling::Off,
                    PatternCulling::RemoveOverlapping => PAPCulling::RemoveOverlapping,
                    PatternCulling::EraseOverlapping(_, _) => PAPCulling::EraseOverlapping,
                };

                let mut culling_mode = prev_culling_mode.clone();
                egui::ComboBox::from_label("Mode")
                    .selected_text(format!("{:?}", culling_mode))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut culling_mode, PAPCulling::Off, "Off");
                        ui.selectable_value(&mut culling_mode, PAPCulling::RemoveOverlapping, "Remove Overlap");
                        ui.selectable_value(&mut culling_mode, PAPCulling::EraseOverlapping, "Erase Overlap");
                    });

                if culling_mode != prev_culling_mode {
                    match culling_mode {
                        PAPCulling::Off => data.prevent_overdraw = PatternCulling::Off,
                        PAPCulling::RemoveOverlapping => data.prevent_overdraw = PatternCulling::RemoveOverlapping,
                        PAPCulling::EraseOverlapping => data.prevent_overdraw = PatternCulling::EraseOverlapping(5., 25.),
                    }
                }

                data.prevent_overdraw = match data.prevent_overdraw {
                    PatternCulling::EraseOverlapping(radius, mut cull_area_percent) => {
                        let mut radius = radius as f32;
                        ui.add(egui::Slider::new(&mut radius, 0.0..=20.0)
                            .text("Radius")
                        );
                        ui.add(egui::Slider::new(&mut cull_area_percent, 0.0..=100.0)
                            .text("Cull Area %"));

                        PatternCulling::EraseOverlapping(radius as f64, cull_area_percent as f64)
                    }
                    PatternCulling::RemoveOverlapping => PatternCulling::RemoveOverlapping,
                    PatternCulling::Off => PatternCulling::Off,
                }
            });

            ui.checkbox(&mut data.warp_pattern, "Warp");
            ui.checkbox(&mut data.split_path, "Split at Discontinuities");
            ui.checkbox(&mut data.center_pattern, "Center");

            egui::ComboBox::from_label("Stretch")
            .selected_text(format!("{:?}", data.stretch))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut data.stretch, PatternStretch::Off, "Off");
                ui.selectable_value(&mut data.stretch, PatternStretch::On, "On");
                ui.selectable_value(&mut data.stretch, PatternStretch::Spacing, "Spacing");
            });

            ui.checkbox(&mut data.simplify, "Simplify");

            ui.add(egui::Slider::new(&mut data.spacing, -20.0..=20.0)
                .text("Spacing")
            );

            ui.add(egui::Slider::new(&mut data.normal_offset, -20.0..=20.0)
                .text("Normal Offset")
            );

            ui.add(egui::Slider::new(&mut data.tangent_offset, -20.0..=20.0)
                .text("Tangent Offset")
            );

            ui.add(egui::Slider::new(&mut data.pattern_scale.0, -3.0..=3.0)
                .text("Scale X")
            );


            ui.add(egui::Slider::new(&mut data.pattern_scale.1, -3.0..=3.0)
                .text("Scale Y")
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
                            prevent_overdraw: PatternCulling::Off,
                            two_pass_culling: false,
                            reverse_path: false,
                            reverse_culling: false,
                            warp_pattern: true,
                            split_path: false,
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
