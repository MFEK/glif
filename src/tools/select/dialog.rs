use super::Select;

use crate::editor::Editor;
use crate::user_interface::Interface;


const DIALOG_ADDITIONAL_HEIGHT: f32 = 150.;

// Make dialog box at right
impl Select {
    pub fn select_settings(&self, v: &mut Editor, i: &Interface, ui: &egui::Ui) {
        /*
        let (ci, pi) = if let Some((ci, pi)) = v.selected_point() {
            (ci, pi)
        } else {
            return;
        };

        let layer = v.get_active_layer_ref();
        if v.get_active_layer_ref().outline[ci].get_type() != MFEKContourInnerType::Cubic {
            return
        }
        let point = get_point!(layer, ci, pi).unwrap().cubic().unwrap().clone();

        let multiple_points_selected = v.selected.len() > 1;

        let (tx, ty, tw, th) = i.get_tools_dialog_rect();
        let mut should_clear_contour_op = false;
        let mut should_apply_contour_op = false;
        let on_open_contour = is_contour_open!(v.get_active_layer_ref(), ci);
        let contour_len = get_contour_len!(v.get_active_layer_ref(), ci);
        let on_last_open_point: bool = pi == contour_len - 1 && on_open_contour;
        let on_first_open_point: bool = pi == 0 && on_open_contour;

        let mut new_point: Point<MFEKPointData> = point.clone();
        let mut pname = imgui::ImString::from(
            new_point
                .name
                .as_ref()
                .map(|n| n.to_string())
                .unwrap_or_else(String::new),
        );
        pname.reserve(IMGUI_RESERVE);

        imgui::Window::new(&if multiple_points_selected {
            imgui::ImString::new("Points")
        } else {
            imgui::im_str!("Point @({}, {}) of type {:?}", ci, pi, new_point.ptype)
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
            imgui_decimal_text_field("X", ui, &mut new_point.x, None);
            // Y
            imgui_decimal_text_field("Y", ui, &mut new_point.y, None);

            let mut a_colocated = new_point.a == Handle::Colocated;
            let mut b_colocated = new_point.b == Handle::Colocated;
            // A (next)
            if !on_last_open_point {
                ui.text(imgui::im_str!("Next off-curve point"));
                ui.checkbox(imgui::im_str!("A Colocated"), &mut a_colocated);
                // AX
                let (mut ax, mut ay) = new_point.handle_or_colocated(WhichHandle::A, &|f| f, &|f| f);
                let orig_axy = (ax, ay);
                imgui_decimal_text_field("AX", ui, &mut ax, None);
                // AY
                imgui_decimal_text_field("AY", ui, &mut ay, None);

                if (ax, ay) != orig_axy {
                    new_point.a = Handle::At(ax, ay);
                    new_point.ptype = PointType::Curve;
                } else if a_colocated {
                    new_point.a = Handle::Colocated;
                }
                // Ar, AΘ
                imgui_radius_theta("A", ui, WhichHandle::A, &mut new_point);
            }

            // B (prev)
            if !on_first_open_point {
                ui.text(imgui::im_str!("Previous off-curve point"));
                ui.checkbox(imgui::im_str!("B Colocated"), &mut b_colocated);
                // BX
                let (mut bx, mut by) = new_point.handle_or_colocated(WhichHandle::B, &|f| f, &|f| f);
                let orig_bxy = (bx, by);
                imgui_decimal_text_field("BX", ui, &mut bx, None);
                // BY
                imgui_decimal_text_field("BY", ui, &mut by, None);
                if (bx, by) != orig_bxy {
                    new_point.b = Handle::At(bx, by);
                    new_point.ptype = PointType::Curve;
                } else if b_colocated {
                    new_point.b = Handle::Colocated;
                }
                // Br, BΘ
                imgui_radius_theta("B", ui, WhichHandle::B, &mut new_point);
            }

            let name_field = ui
                .input_text(imgui::im_str!("Name"), &mut pname)
                .enter_returns_true(true);
            if name_field.build() {
                if pname.to_str().len() > 0 {
                    new_point.name = Some(pname.to_string());
                } else {
                    new_point.name = None;
                }
            }

            if v.get_active_layer_ref().outline[ci].operation().is_some() {
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

        if point.ptype == PointType::Move {
            new_point.ptype = PointType::Move;
        }

        if should_clear_contour_op {
            v.begin_modification("Reset contour op.");
            v.get_active_layer_mut().outline[ci].set_operation(None);
            v.end_modification();
        }

        if should_apply_contour_op {
            v.begin_modification("Apply contour op.");
            {
                let layer = v.get_active_layer_mut();
                let op = &layer.outline[ci].operation().clone();
                layer.outline[ci].set_operation(None);
                let ol = op.build(&layer.outline[ci]);
                layer.outline.remove(ci);
                for contour in ol {
                    layer.outline.push(contour);
                }
            };
            v.contour_idx = None;
            v.point_idx = None;
            v.selected = HashSet::new();
            v.end_modification();
        }

        if point.x != new_point.x
            || point.y != new_point.y
            || point.a != new_point.a
            || point.b != new_point.b
            || point.name != new_point.name
            || point.ptype != new_point.ptype
        {
            v.begin_modification("Point properties changed (dialog)");
            {
                let layer = v.get_active_layer_mut();
                match &mut layer.outline[ci].inner_mut() {
                    MFEKContourInner::Cubic(contour) => contour[pi] = new_point,
                    _ => panic!("Unsupported")
                } 
            }
            v.end_modification();
        }

        // unsafe function! OK here as these handles are always invalid and if we used history
        // version then it would be an invalid begin_modification() (from dialog) inside a
        // begin_modification() (from moving handles).
        v.with_active_layer_mut_no_history(|layer| {
            if on_first_open_point {
                match &mut layer.outline[ci].inner_mut() {
                    MFEKContourInner::Cubic(contour) => contour[pi].b = Handle::Colocated,
                    _ => unreachable!()
                } 
            } else if on_last_open_point {
                match &mut layer.outline[ci].inner_mut() {
                    MFEKContourInner::Cubic(contour) => contour[pi].a = Handle::Colocated,
                    _ => unreachable!()
                }             }
        });
        */
    }
}
