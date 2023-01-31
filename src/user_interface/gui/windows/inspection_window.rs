use egui::{Context, Ui};
use crate::{editor::Editor, user_interface::{Interface, gui::window::GlifWindow}, get_point};
use glifparser::{glif::{contour::MFEKContourCommon, point::MFEKPointCommon}, Point, MFEKPointData, PointData, WhichHandle, Handle};

pub struct InspectionWindow {
    // is this window open?
    open: bool,
}

impl InspectionWindow {
    pub fn new() -> Self {
        InspectionWindow { open: false }
    }
}

impl GlifWindow for InspectionWindow {
    fn open(&self) -> bool {
        self.open
    }

    fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    fn build(&mut self, ctx: &Context, v: &mut Editor, i: &mut Interface) {
        egui::Window::new("Inspector")
            .resizable(true)
            .collapsible(true)
            .open(&mut self.open)
            .enabled(!v.is_modifying())
            .default_width(64.)
            .default_pos([1000000., 25.])
            .constrain(true)
            .show(ctx, |ui| {
                if let Some(selected_point) = v.selected_point() {
                    let mut contour = v.get_active_layer_ref().outline[selected_point.0].clone();
                    let  point = contour.get_point_mut(selected_point.1).expect("Editor should have valid selection!");
                    // do contour stuff

                    ui.collapsing("Contour", |ui| {
                        let mut contour = v.get_active_layer_ref().outline[selected_point.0].clone();

                        ui.label(format!("Type: {:?}", contour.get_type()));
                        
                        if let Some(op) = contour.operation() {
                            ui.label(format!("Operation: {:?}", op));
                            if ui.button("Apply Contour Operation").clicked() {
                                
                            }
                        }

                        let mut open = contour.is_open();
                        ui.checkbox(&mut open, "Open");

                        if open != contour.is_open() {
                            if open {
                                contour.set_open();
                            } else {
                                contour.set_closed();
                            }

                            v.begin_modification("Modified contour with inspector.", true);
                            v.get_active_layer_mut().outline[selected_point.0] = contour;
                            v.end_modification();
                        }
                    });
                    ui.collapsing("Point", |ui| {
                        ui.label("Position:");
                        point.set_position(egui_coordinate_helper(ui, point.x()), egui_coordinate_helper(ui, point.y()));
    
                        ui.collapsing("Handles", |ui| {
                            if point.has_handle(WhichHandle::A) {
                                ui.label("Handle A:");
                                if let Some(handle_a_pos) = point.get_handle_position(WhichHandle::A) {
                                    point.set_handle_position(WhichHandle::A,
                                        egui_coordinate_helper(ui, handle_a_pos.0), 
                                        egui_coordinate_helper(ui, handle_a_pos.1)
                                    )
                                }
    
                                let prev_checked = point.get_handle(WhichHandle::A).unwrap() == Handle::Colocated;
                                let mut checked = point.get_handle(WhichHandle::A).unwrap() == Handle::Colocated;
                                ui.checkbox(&mut checked, "Colocated A");
    
                                if prev_checked != checked {
                                    match checked {
                                        false => {
                                            let (x, y) = point.get_position();
                                            point.set_handle_position(WhichHandle::A, x, y);
                                        },
                                        true => {
                                            point.colocate_handle(WhichHandle::A);
                                        }
                                    }
                                }
                            }
    
                            if point.has_handle(WhichHandle::B) {
                                ui.label("Handle B:");
                                if let Some(handle_b_pos) = point.get_handle_position(WhichHandle::B) {
                                    point.set_handle_position(WhichHandle::B,
                                        egui_coordinate_helper(ui, handle_b_pos.0), 
                                        egui_coordinate_helper(ui, handle_b_pos.1)
                                    )
                                }
                                let prev_checked = point.get_handle(WhichHandle::B).unwrap() == Handle::Colocated;
                                let mut checked = point.get_handle(WhichHandle::B).unwrap() == Handle::Colocated;
                                ui.checkbox(&mut checked, "Colocated B");
    
                                if prev_checked != checked {
                                    match checked {
                                        false => {
                                            let (x, y) = point.get_position();
                                            point.set_handle_position(WhichHandle::B, x, y);
                                        },
                                        true => {
                                            point.colocate_handle(WhichHandle::B);
                                        }
                                    }
                                }
                            }
                        });
    
    
                        if !point_equivalent(point, v.get_active_layer_ref().outline[selected_point.0].get_point(selected_point.1).unwrap()) {
                            v.begin_modification("Modified point with inspector.", true);
    
                            let mutated_point = v.get_active_layer_mut().outline[selected_point.0].get_point_mut(selected_point.1).unwrap();
                            let (x, y) = point.get_position();
                            mutated_point.set_position(x, y);
    
                            if let Some(handle_a) = point.get_handle(WhichHandle::A) {
                                mutated_point.set_handle(WhichHandle::A, handle_a);
                            }
    
                            if let Some(handle_b) = point.get_handle(WhichHandle::B) {
                                mutated_point.set_handle(WhichHandle::B, handle_b);
                            }
                            
                            if let Some(name) = point.get_name() {
                                mutated_point.set_name(name);
                            }
    
                            v.end_modification();
                        }
                    });
                } else {
                    ui.label("No point selected to inspect!");
                }
            });
    }
}

fn point_equivalent<PD:PointData>(a: &dyn MFEKPointCommon<PD>, b: &dyn MFEKPointCommon<PD>) -> bool {
    a.get_name() == b.get_name() &&
    a.get_position() == b.get_position() &&
    a.get_handle_position(WhichHandle::A) == b.get_handle_position(WhichHandle::A) &&
    a.get_handle_position(WhichHandle::B) == b.get_handle_position(WhichHandle::B)
}
fn egui_coordinate_helper(ui: &mut Ui, coord: f32) -> f32 {
    let mut coord_string = coord.to_string();
    if ui.text_edit_singleline(&mut coord_string).changed() {
        if let Ok(x) = coord_string.parse::<f32>() {
            return x
        }
    }

    coord
}
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