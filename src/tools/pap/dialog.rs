use super::super::prelude::*;
use super::PAP;
use crate::user_interface::Interface;
use glifparser::glif::PatternStretch;
use glifparser::glif::{ContourOperations, PatternCopies, PatternSubdivide};
use imgui::Ui;

fn imgui_decimal_text_field(label: &str, ui: &imgui::Ui, data: &mut f32) {
    let mut x = format!("{}", data);
    let label = String::from(label);
    let entered;
    {
        let it = ui.input_text(&label, &mut x);
        entered = it
            .enter_returns_true(true)
            .chars_decimal(true)
            .chars_noblank(true)
            .auto_select_all(true)
            .build();
    }
    if entered {
        if x.as_str().len() > 0 {
            let new_x: f32 = x.as_str().parse().unwrap();
            *data = new_x;
        }
    }
}

fn repeat_type_to_idx(rt: PatternCopies) -> usize {
    match rt {
        PatternCopies::Single => 0,
        PatternCopies::Repeated => 1,
        _ => unreachable!(),
    }
}

fn idx_to_repeat_type(idx: usize) -> PatternCopies {
    match idx {
        0 => PatternCopies::Single,
        1 => PatternCopies::Repeated,
        _ => unreachable!(),
    }
}

fn idx_to_stretch_type(idx: usize) -> PatternStretch {
    match idx {
        0 => PatternStretch::Off,
        1 => PatternStretch::On,
        2 => PatternStretch::Spacing,
        _ => unreachable!()
    }
}


fn stretch_type_to_idx(idx: PatternStretch) -> usize {
    match idx {
        PatternStretch::Off => 0,
        PatternStretch::On => 1,
        PatternStretch::Spacing => 2,
    }
}

impl PAP {
    pub fn tool_dialog(&mut self, v: &mut Editor, i: &Interface, ui: &Ui) {
        let (tx, ty, tw, th) = i.get_tools_dialog_rect();

        imgui::Window::new(&ui, &imgui::ImString::new("Pattern Along Path"))
            .bg_alpha(1.) // See comment on fn redraw_skia
            .flags(
                imgui::WindowFlags::NO_RESIZE
                    | imgui::WindowFlags::NO_MOVE
                    | imgui::WindowFlags::NO_COLLAPSE,
            )
            .position([tx, ty], imgui::Condition::Always)
            .size([tw, th], imgui::Condition::Always)
            .build(|| {
                if v.contour_idx.is_none() {
                    return;
                };
                let contour_idx = v.contour_idx.unwrap();

                let operation =
                    v.with_active_layer(|layer| layer.outline[contour_idx].operation.clone());

                match operation {
                    // TODO: Clean this up. I could reduce the number of lines here by a lot if I make a few changes to how the function works,
                    // and add imgui utility functions for sliders and checkboxes.
                    Some(ContourOperations::PatternAlongPath { data }) => {
                        let old_repeat = repeat_type_to_idx(data.copies.clone());
                        let mut new_repeat = old_repeat;

                        let options = ["Single", "Repeated"];

                        ui.combo_simple_string("Mode", &mut new_repeat, &options);

                        let repeat_selection = idx_to_repeat_type(new_repeat);

                        // we only update the contour and previews when our selection changes
                        if old_repeat != new_repeat {
                            let mut new_data = data.clone();
                            new_data.copies = repeat_selection;
                            let new_op = ContourOperations::PatternAlongPath { data: new_data };

                            v.begin_modification("PAP dialog modification.");
                            v.with_active_layer_mut(|layer| {
                                layer.outline[contour_idx].operation = Some(new_op.clone())
                            });
                            v.end_modification();
                        }

                        let options = [
                            "0",
                            "1",
                            "2",
                            "3",
                        ];

                        let cur_subdivisions = match data.subdivide {
                            PatternSubdivide::Simple(times) => times,
                            _ => 0,
                        };

                        let mut new_subdivisions = cur_subdivisions;

                        ui.combo_simple_string("Subdivisions", &mut new_subdivisions, &options);

                        if cur_subdivisions != new_subdivisions {
                            let mut new_data = data.clone();
                            new_data.subdivide = if new_subdivisions == 0 {
                                PatternSubdivide::Off
                            } else {
                                PatternSubdivide::Simple(new_subdivisions)
                            };
                            let new_op = ContourOperations::PatternAlongPath { data: new_data };

                            v.begin_modification("PAP dialog modification.");
                            v.with_active_layer_mut(|layer| {
                                layer.outline[contour_idx].operation = Some(new_op.clone())
                            });
                            v.end_modification();
                        }

                        let mut new_center = data.center_pattern;
                        ui.checkbox("Center", &mut new_center);
                        if new_center != data.center_pattern {
                            let mut new_data = data.clone();
                            new_data.center_pattern = new_center;
                            let new_op = ContourOperations::PatternAlongPath { data: new_data };

                            v.begin_modification("PAP dialog modification.");
                            v.with_active_layer_mut(|layer| {
                                layer.outline[contour_idx].operation = Some(new_op.clone())
                            });
                            v.end_modification();
                        }

                        let options = [
                            "Off",
                            "On",
                            "Spacing",
                        ];
            
                        let mut current_selection = stretch_type_to_idx(data.stretch);
                        
                        ui.combo_simple_string("Stretch", &mut current_selection, &options);

                        let old_stretch = data.stretch;
                        let new_stretch = idx_to_stretch_type(current_selection);
                        if old_stretch != new_stretch {
                            let mut new_data = data.clone();
                            new_data.stretch = new_stretch;
                            let new_op = ContourOperations::PatternAlongPath { data: new_data };

                            v.begin_modification("PAP dialog modification.");
                            v.with_active_layer_mut(|layer| {
                                layer.outline[contour_idx].operation = Some(new_op.clone())
                            });
                            v.end_modification();
                        }

                        let old_simplify = data.simplify;
                        let mut new_simplify = old_simplify;
                        ui.checkbox("Simplify", &mut new_simplify);
                        if old_simplify != new_simplify {
                            let mut new_data = data.clone();
                            new_data.simplify = new_simplify;
                            let new_op = ContourOperations::PatternAlongPath { data: new_data };

                            v.begin_modification("PAP dialog modification.");
                            v.with_active_layer_mut(|layer| {
                                layer.outline[contour_idx].operation = Some(new_op.clone())
                            });
                            v.end_modification();
                        }

                        let mut new_spacing = data.spacing as f32;
                        imgui_decimal_text_field("Spacing", ui, &mut new_spacing);

                        if new_spacing != data.spacing as f32 {
                            let mut new_data = data.clone();
                            new_data.spacing = new_spacing as f64;
                            let new_op = ContourOperations::PatternAlongPath { data: new_data };

                            v.begin_modification("PAP dialog modification.");
                            v.with_active_layer_mut(|layer| {
                                layer.outline[contour_idx].operation = Some(new_op.clone())
                            });
                            v.end_modification();
                        }

                        let mut new_normal = data.normal_offset as f32;
                        imgui_decimal_text_field("Normal Offset", ui, &mut new_normal);

                        if new_normal != data.normal_offset as f32 {
                            let mut new_data = data.clone();
                            new_data.normal_offset = new_normal as f64;
                            let new_op = ContourOperations::PatternAlongPath { data: new_data };

                            v.begin_modification("PAP dialog modification.");
                            v.with_active_layer_mut(|layer| {
                                layer.outline[contour_idx].operation = Some(new_op.clone())
                            });
                            v.end_modification();
                        }

                        let mut new_tangent = data.tangent_offset as f32;
                        imgui_decimal_text_field("Tangent Offset", ui, &mut new_tangent);

                        if new_tangent != data.tangent_offset as f32 {
                            let mut new_data = data.clone();
                            new_data.tangent_offset = new_tangent as f64;
                            let new_op = ContourOperations::PatternAlongPath { data: new_data };

                            v.begin_modification("PAP dialog modification.");
                            v.with_active_layer_mut(|layer| {
                                layer.outline[contour_idx].operation = Some(new_op.clone())
                            });
                            v.end_modification();
                        }

                        let mut new_x_scale = data.pattern_scale.0 as f32;
                        let mut new_y_scale = data.pattern_scale.1 as f32;
                        imgui_decimal_text_field("##", ui, &mut new_x_scale);
                        ui.same_line();
                        imgui_decimal_text_field("Scale", ui, &mut new_y_scale);

                        if (new_x_scale as f64, new_y_scale as f64) != data.pattern_scale {
                            let mut new_data = data.clone();
                            new_data.pattern_scale = (new_x_scale as f64, new_y_scale as f64);
                            let new_op = ContourOperations::PatternAlongPath { data: new_data };

                            v.begin_modification("PAP dialog modification.");
                            v.with_active_layer_mut(|layer| {
                                layer.outline[contour_idx].operation = Some(new_op.clone())
                            });
                            v.end_modification();
                        }

                        let mut overdraw = data.prevent_overdraw;
                        imgui::Slider::new("Prevent Overdraw", 0., 1.)
                            .build(ui, &mut overdraw);

                        if overdraw != data.prevent_overdraw {
                            let mut new_data = data.clone();
                            new_data.prevent_overdraw = overdraw;
                            let new_op = ContourOperations::PatternAlongPath { data: new_data };

                            v.begin_modification("PAP overdraw changed.");
                            v.with_active_layer_mut(|layer| {
                                layer.outline[contour_idx].operation = Some(new_op.clone())
                            });
                            v.end_modification();
                            v.collapse_history_entries();
                        }
                        
                        let old_twopass = data.two_pass_culling;
                        let mut new_twopass = old_twopass;
                        ui.checkbox("Two-pass Culling", &mut new_twopass);
                        if old_twopass != new_twopass {
                            let mut new_data = data.clone();
                            new_data.two_pass_culling = new_twopass;
                            let new_op = ContourOperations::PatternAlongPath { data: new_data };

                            v.begin_modification("PAP dialog modification.");
                            v.with_active_layer_mut(|layer| {
                                layer.outline[contour_idx].operation = Some(new_op.clone())
                            });
                            v.end_modification();
                        }

                        let mut reverse_culling = data.reverse_culling;
                        ui.checkbox("Reverse Culling", &mut reverse_culling);
                        if reverse_culling != data.reverse_culling {
                            let mut new_data = data.clone();
                            new_data.reverse_culling = reverse_culling;
                            let new_op = ContourOperations::PatternAlongPath { data: new_data };

                            v.begin_modification("PAP dialog modification.");
                            v.with_active_layer_mut(|layer| {
                                layer.outline[contour_idx].operation = Some(new_op.clone())
                            });
                            v.end_modification();
                        }

                        let mut reverse_path = data.reverse_path;
                        ui.checkbox("Reverse", &mut reverse_path);

                        if reverse_path != data.reverse_path {
                            let mut new_data = data.clone();
                            new_data.reverse_path = reverse_path;
                            let new_op = ContourOperations::PatternAlongPath { data: new_data };

                            v.begin_modification("PAP dialog modification.");
                            v.with_active_layer_mut(|layer| {
                                layer.outline[contour_idx].operation = Some(new_op.clone())
                            });
                            v.end_modification();
                        }
                        
                    }
                    _ => {}
                }
            });
    }
}
