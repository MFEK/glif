use super::super::prelude::*;
use super::Dash;
use crate::user_interface::{self, Interface};
use glifparser::glif::{ContourOperations, DashCull};
use imgui::Ui;
use skulpin::skia_safe::{PaintCap, PaintJoin};

impl Dash {
    pub fn tool_dialog(&mut self, v: &mut Editor, i: &Interface, ui: &Ui) {
        let (tx, ty, tw, th) = i.get_tools_dialog_rect();

        imgui::Window::new(&imgui::ImString::new("Dash Along Path"))
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

                if let Some(ContourOperations::DashAlongPath { data }) = operation {
                    let mut new_width = data.stroke_width as f32;
                    user_interface::util::imgui_decimal_text_field(
                        "Stroke Width",
                        ui,
                        &mut new_width,
                        None,
                    );
                    let label = imgui::ImString::new("Dashes");
                    let mut dashes = imgui::im_str!(
                        "{}",
                        data.dash_desc
                            .iter()
                            .map(|f| f.to_string())
                            .collect::<Vec<_>>()
                            .join(" ")
                    );
                    dashes.reserve(32);
                    let mut new_dash_desc = data.dash_desc.clone();
                    let entered;
                    {
                        let it = ui.input_text(&label, &mut dashes);
                        entered = it.enter_returns_true(true).auto_select_all(true).build();
                    }
                    if entered && !dashes.to_str().is_empty() {
                        let new = dashes
                            .to_str()
                            .split(" ")
                            .map(|s| s.parse::<f32>())
                            .filter(|f| f.is_ok())
                            .map(|o| o.unwrap())
                            .collect::<Vec<_>>();
                        if new.len() > 1 && new.len() % 2 == 0 {
                            new_dash_desc = new;
                        }
                    }

                    let do_cull = data.cull.is_some();
                    let mut new_do_cull = do_cull;
                    ui.checkbox(imgui::im_str!("Cull"), &mut new_do_cull);

                    let include_last_path = data.include_last_path;
                    let mut new_include_last_path = include_last_path;
                    if do_cull {
                        ui.checkbox(imgui::im_str!("Include last?"), &mut new_include_last_path);
                    }

                    let cull_width = data.cull.map(|c| c.width as f32).unwrap_or(0.);
                    let mut new_cull_width = cull_width;
                    if do_cull {
                        user_interface::util::imgui_decimal_text_field(
                            "Cull Width",
                            ui,
                            &mut new_cull_width,
                            None,
                        );
                    }

                    let cull_area_cutoff = data.cull.map(|c| c.area_cutoff as f32).unwrap_or(0.);
                    let mut new_cull_area_cutoff = cull_area_cutoff;
                    if do_cull {
                        user_interface::util::imgui_decimal_text_field(
                            "Cull Area Cutoff",
                            ui,
                            &mut new_cull_area_cutoff,
                            None,
                        );
                    }

                    const JOIN_OPTIONS: [&imgui::ImStr; 3] = [
                        imgui::im_str!("Miter"),
                        imgui::im_str!("Bevel"),
                        imgui::im_str!("Round"),
                    ];
                    const JOINS: [u8; 3] = [
                        PaintJoin::Miter as u8,
                        PaintJoin::Bevel as u8,
                        PaintJoin::Round as u8,
                    ];
                    let join = JOINS.iter().position(|&r| r == data.paint_join).unwrap();
                    let mut new_join = join;
                    imgui::ComboBox::new(imgui::im_str!("Dash Join")).build_simple_string(
                        ui,
                        &mut new_join,
                        &JOIN_OPTIONS,
                    );

                    const CAP_OPTIONS: [&imgui::ImStr; 3] = [
                        imgui::im_str!("Butt"),
                        imgui::im_str!("Square"),
                        imgui::im_str!("Round"),
                    ];
                    const CAPS: [u8; 3] = [
                        PaintCap::Butt as u8,
                        PaintCap::Square as u8,
                        PaintCap::Round as u8,
                    ];
                    let cap = CAPS.iter().position(|&r| r == data.paint_cap).unwrap();
                    let mut new_cap = cap;
                    imgui::ComboBox::new(imgui::im_str!("Dash Cap")).build_simple_string(
                        ui,
                        &mut new_cap,
                        &CAP_OPTIONS,
                    );

                    if new_width != data.stroke_width
                        || new_dash_desc != data.dash_desc
                        || new_do_cull != do_cull
                        || cull_width != new_cull_width
                        || cull_area_cutoff != new_cull_area_cutoff
                        || join != new_join
                        || cap != new_cap
                        || include_last_path != new_include_last_path
                    {
                        let mut new_data = data.clone();
                        new_data.stroke_width = new_width;
                        new_data.dash_desc = new_dash_desc;
                        new_data.cull = if new_do_cull {
                            Some(DashCull {
                                width: new_cull_width,
                                area_cutoff: new_cull_area_cutoff,
                            })
                        } else {
                            None
                        };
                        new_data.paint_join = JOINS[new_join];
                        new_data.paint_cap = CAPS[new_cap];
                        new_data.include_last_path = new_include_last_path;
                        let new_op = ContourOperations::DashAlongPath { data: new_data };

                        v.begin_modification("Dash dialog modification.");
                        v.with_active_layer_mut(|layer| {
                            layer.outline[contour_idx].operation = Some(new_op.clone())
                        });
                        v.end_modification();
                    }
                }
            });
    }
}
