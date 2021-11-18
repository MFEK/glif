use super::super::prelude::*;
use super::PAP;
use crate::user_interface::{self, Interface};
use glifparser::glif::PatternStretch;
use glifparser::glif::{ContourOperations, PatternCopies, PatternSubdivide};
use imgui::Ui;

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
        _ => unreachable!(),
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

        imgui::Window::new(&imgui::ImString::new("Pattern Along Path"))
            .bg_alpha(1.) // See comment on fn redraw_skia
            .flags(
                imgui::WindowFlags::NO_RESIZE
                    | imgui::WindowFlags::NO_MOVE
                    | imgui::WindowFlags::NO_COLLAPSE,
            )
            .position([tx, ty], imgui::Condition::Always)
            .size([tw, th], imgui::Condition::Always)
            .build(ui, || {
                if v.contour_idx.is_none() {
                    return;
                };
                let contour_idx = v.contour_idx.unwrap();

                let operation =
                    v.with_active_layer(|layer| layer.outline[contour_idx].operation.clone());

                // TODO: Clean this up. I could reduce the number of lines here by a lot if I make a few changes to how the function works,
                // and add imgui utility functions for sliders and checkboxes.
                if let Some(ContourOperations::PatternAlongPath { data }) = operation {
                    let old_repeat = repeat_type_to_idx(data.copies.clone());
                    let mut new_repeat = old_repeat;

                    let options = [imgui::im_str!("Single"), imgui::im_str!("Repeated")];

                    imgui::ComboBox::new(imgui::im_str!("Mode")).build_simple_string(
                        ui,
                        &mut new_repeat,
                        &options,
                    );

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
                        imgui::im_str!("0"),
                        imgui::im_str!("1"),
                        imgui::im_str!("2"),
                        imgui::im_str!("3"),
                    ];

                    let cur_subdivisions = match data.subdivide {
                        PatternSubdivide::Simple(times) => times,
                        _ => 0,
                    };

                    let mut new_subdivisions = cur_subdivisions;

                    imgui::ComboBox::new(imgui::im_str!("Subdivisions")).build_simple_string(
                        ui,
                        &mut new_subdivisions,
                        &options,
                    );

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
                    ui.checkbox(imgui::im_str!("Center"), &mut new_center);
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
                        imgui::im_str!("Off"),
                        imgui::im_str!("On"),
                        imgui::im_str!("Spacing"),
                    ];

                    let mut current_selection = stretch_type_to_idx(data.stretch);
                    imgui::ComboBox::new(imgui::im_str!("Stretch")).build_simple_string(
                        ui,
                        &mut current_selection,
                        &options,
                    );

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
                    ui.checkbox(imgui::im_str!("Simplify"), &mut new_simplify);
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
                    user_interface::util::imgui_decimal_text_field(
                        "Spacing",
                        ui,
                        &mut new_spacing,
                        None,
                    );

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
                    user_interface::util::imgui_decimal_text_field(
                        "Normal Offset",
                        ui,
                        &mut new_normal,
                        None,
                    );

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
                    user_interface::util::imgui_decimal_text_field(
                        "Tangent Offset",
                        ui,
                        &mut new_tangent,
                        None,
                    );

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
                    user_interface::util::imgui_decimal_text_field(
                        "##",
                        ui,
                        &mut new_x_scale,
                        None,
                    );
                    ui.same_line(0.);
                    user_interface::util::imgui_decimal_text_field(
                        "Scale",
                        ui,
                        &mut new_y_scale,
                        None,
                    );

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
                    imgui::Slider::new(imgui::im_str!("Prevent Overdraw"))
                        .range(0. ..=1.)
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
                    ui.checkbox(imgui::im_str!("Two-pass Culling"), &mut new_twopass);
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
                    ui.checkbox(imgui::im_str!("Reverse Culling"), &mut reverse_culling);
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
                    ui.checkbox(imgui::im_str!("Reverse"), &mut reverse_path);

                    if reverse_path != data.reverse_path {
                        let mut new_data = data;
                        new_data.reverse_path = reverse_path;
                        let new_op = ContourOperations::PatternAlongPath { data: new_data };

                        v.begin_modification("PAP dialog modification.");
                        v.with_active_layer_mut(|layer| {
                            layer.outline[contour_idx].operation = Some(new_op.clone())
                        });
                        v.end_modification();
                    }
                }
            });
    }
}
