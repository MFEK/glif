use std::collections::HashSet;
use std::f32::consts::PI;

use super::Select;

use crate::contour_operations::ContourOperation;
use crate::editor::macros::{get_contour_len, get_contour_type, get_point};
use crate::editor::Editor;
use crate::user_interface::util::{imgui_decimal_text_field, imgui_radius_theta};
use crate::user_interface::Interface;
use crate::util::MFEKGlifPointData;

use glifparser::glif::{ContourOperations, MFEKOutline};
use glifparser::{Handle, Point, PointType, WhichHandle};
use MFEKmath::polar::PolarCoordinates;

use imgui;

const DIALOG_ADDITIONAL_HEIGHT: f32 = 150.;

// Make dialog box at right
impl Select {
    pub fn select_settings(&self, v: &mut Editor, i: &Interface, ui: &imgui::Ui) {
        let (ci, pi) = if let Some((ci, pi)) = v.selected() {
            (ci, pi)
        } else {
            return;
        };

        let multiple_points_selected = v.selected.len() > 1;

        let (tx, ty, tw, th) = i.get_tools_dialog_rect();
        let mut orig_point: Point<_> = Point::new();
        let mut point: Point<MFEKGlifPointData> = Point::new();
        let mut should_make_next_point_curve: bool = false;
        let mut should_clear_contour_op = false;
        let mut should_apply_contour_op = false;
        let on_open_contour = v.with_active_layer(|l| get_contour_type!(l, ci) == PointType::Move);
        let contour_len = v.with_active_layer(|l| get_contour_len!(l, ci));
        v.with_active_layer(|layer| {
            point = get_point!(layer, ci, pi).clone();
            orig_point = point.clone();

            let on_last_open_point: bool = pi == contour_len - 1 && on_open_contour;
            let on_first_open_point: bool = pi == 0 && on_open_contour;

            imgui::Window::new(&if multiple_points_selected {
                imgui::ImString::new("Points")
            } else {
                imgui::im_str!("Point ({}, {})", ci, pi)
            })
            .bg_alpha(1.) // See comment on fn redraw_skia
            .flags(
                imgui::WindowFlags::NO_RESIZE
                    | imgui::WindowFlags::NO_MOVE
                    | imgui::WindowFlags::NO_COLLAPSE,
            )
            .position(
                [tx, ty - DIALOG_ADDITIONAL_HEIGHT],
                imgui::Condition::Always,
            )
            .size(
                [tw, th + DIALOG_ADDITIONAL_HEIGHT],
                imgui::Condition::Always,
            )
            .build(ui, || {
                if multiple_points_selected {
                    ui.text(imgui::im_str!("Multiple points selected"));
                    return;
                }

                // X
                imgui_decimal_text_field("X", ui, &mut point.x, None);
                // Y
                imgui_decimal_text_field("Y", ui, &mut point.y, None);

                // A
                let mut a_colocated = point.a == Handle::Colocated;
                if !on_last_open_point {
                    ui.text(imgui::im_str!("Next off-curve point"));
                    ui.checkbox(imgui::im_str!("A Colocated"), &mut a_colocated);
                    // AX
                    let (mut ax, mut ay) = point.handle_or_colocated(WhichHandle::A, |f| f, |f| f);
                    let orig_axy = (ax, ay);
                    imgui_decimal_text_field("AX", ui, &mut ax, None);
                    // AY
                    imgui_decimal_text_field("AY", ui, &mut ay, None);

                    if (ax, ay) != orig_axy {
                        point.a = Handle::At(ax, ay);
                        point.ptype = PointType::Curve;
                        should_make_next_point_curve = true;
                    } else if a_colocated {
                        point.a = Handle::Colocated;
                    }
                    // ArΘ
                    let (ar, mut atheta) = point.polar(WhichHandle::A);
                    atheta *= 180. / PI;
                    atheta -= 180.;
                    imgui_radius_theta("A", ui, ar, atheta, WhichHandle::A, &mut point);
                }

                // B
                let mut b_colocated = point.b == Handle::Colocated;
                if !on_first_open_point {
                    ui.text(imgui::im_str!("Previous off-curve point"));
                    ui.checkbox(imgui::im_str!("B Colocated"), &mut b_colocated);
                    // BX
                    let (mut bx, mut by) = point.handle_or_colocated(WhichHandle::B, |f| f, |f| f);
                    let orig_bxy = (bx, by);
                    imgui_decimal_text_field("BX", ui, &mut bx, None);
                    // BY
                    imgui_decimal_text_field("BY", ui, &mut by, None);
                    if (bx, by) != orig_bxy {
                        point.b = Handle::At(bx, by);
                        point.ptype = PointType::Curve;
                    } else if b_colocated {
                        point.b = Handle::Colocated;
                    }
                    // BrΘ
                    let (br, mut btheta) = point.polar(WhichHandle::B);
                    btheta *= 180. / PI;
                    btheta -= 180.;
                    if btheta.is_sign_positive() {
                        btheta = 360. - btheta;
                    }
                    imgui_radius_theta("B", ui, br, btheta, WhichHandle::B, &mut point);
                }

                if v.with_active_layer(|layer| layer.outline[ci].operation.is_some()) {
                    ui.button(imgui::im_str!("Reset Contour Operation"), [0., 0.]);
                    if ui.is_item_clicked(imgui::MouseButton::Left) {
                        should_clear_contour_op = true;
                    }
                    ui.button(imgui::im_str!("Apply Contour Operation"), [0., 0.]);
                    if ui.is_item_clicked(imgui::MouseButton::Left) {
                        should_apply_contour_op = true;
                    }
                }
            });
        });

        if should_clear_contour_op {
            v.begin_modification("Reset contour op.");
            v.with_active_layer_mut(|layer| layer.outline[ci].operation = None);
            v.end_modification();
        }

        if should_apply_contour_op {
            v.begin_modification("Apply contour op.");
            v.with_active_layer_mut(|layer| {
                let op = &layer.outline[ci].operation.clone();
                layer.outline[ci].operation = None;
                let ol = match op {
                    Some(ContourOperations::DashAlongPath { data }) => {
                        data.build(&layer.outline[ci])
                    }
                    Some(ContourOperations::PatternAlongPath { data }) => {
                        data.build(&layer.outline[ci])
                    }
                    Some(ContourOperations::VariableWidthStroke { data }) => {
                        data.build(&layer.outline[ci])
                    }
                    _ => (MFEKOutline::new()),
                };
                layer.outline.remove(ci);
                for contour in ol {
                    layer.outline.push(contour);
                }
            });
            v.contour_idx = None;
            v.point_idx = None;
            v.selected = HashSet::new();
            v.end_modification();
        }

        if orig_point.x != point.x
            || orig_point.y != point.y
            || orig_point.a != point.a
            || orig_point.b != point.b
            || orig_point.ptype != point.ptype
        {
            v.begin_modification("Point properties changed (dialog)");
            v.with_active_layer_mut(|layer| {
                if get_point!(layer, ci, pi).ptype == PointType::Move {
                    point.ptype = PointType::Move;
                }
                if should_make_next_point_curve {
                    let ppi = if pi == contour_len - 1 { 0 } else { pi + 1 };
                    if !(on_open_contour && ppi == 0) {
                        get_point!(layer, ci, ppi).ptype = PointType::Curve;
                    }
                }
                get_point!(layer, ci, pi) = point.clone();
            });
            v.end_modification();
        }
    }
}
