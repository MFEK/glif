mod modes;

use self::modes::PenMode;
use self::modes::cubic::CubicMode;
use self::modes::hyper::HyperMode;
use self::modes::quad::QuadMode;

use super::prelude::*;

use crate::is_contour_open;
use crate::tool_behaviors::{move_handle::MoveHandle, pan::PanBehavior, zoom_scroll::ZoomScroll};
use crate::user_interface::Interface;
use glifparser::glif::inner::MFEKContourInnerType;
use glifrenderer::points::draw_point;
use glifparser::glif::mfek::contour::MFEKContourCommon;


use editor::util::get_contour_start_or_end;

#[derive(Clone, Debug)]
pub struct Pen {
    mode: MFEKContourInnerType,
    cubic: CubicMode,
    quad: QuadMode,
    hyper: HyperMode,
}


impl Tool for Pen {
    #[rustfmt::skip]
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent { mouse_info, event_type } => match event_type {
                MouseEventType::Pressed => self.mouse_pressed(v, i, mouse_info),
                _ => (),
            }
            EditorEvent::ScrollEvent { .. } => ZoomScroll::default().event(v, i, event),
            _ => {}
        }
    }

    fn ui(&mut self, _v: &mut Editor, _i: &mut Interface, ui: &mut Ui) {
        imgui::Window::new(imgui::im_str!("Points"))
        .bg_alpha(1.)
        .flags(
            imgui::WindowFlags::NO_RESIZE
                | imgui::WindowFlags::NO_MOVE
                | imgui::WindowFlags::NO_COLLAPSE,
        )
        .position(
            [0., 100.],
            imgui::Condition::Always,
        )
        .size(
            [100., 100.],
            imgui::Condition::Always,
        )
        .build(ui, || {
            let options = [
                imgui::im_str!("Cubic"),
                imgui::im_str!("Quad"),
                imgui::im_str!("Hyper"),
            ];

            let mut mode = contour_type_to_string(self.mode.clone());

            imgui::ComboBox::new(imgui::im_str!("Subdivisions")).build_simple_string(
                ui,
                &mut mode,
                &options,
            );

            self.mode = match mode {
                0 => MFEKContourInnerType::Cubic,
                1 => MFEKContourInnerType::Quad,
                2 => MFEKContourInnerType::Hyper,
                _ => unreachable!()
            }
        })
    }

    fn draw(&mut self, v: &Editor, i: &Interface, canvas: &mut Canvas) {
        self.draw_merge_preview(v, i, canvas);
        self.draw_nearest_point(v, i, canvas);
    }
}

impl Pen {
    pub fn new() -> Self {
        Self {
            mode: MFEKContourInnerType::Cubic,
            cubic: CubicMode {  },
            quad: QuadMode {  },
            hyper: HyperMode { },
        }
    }

    fn mouse_pressed(&mut self, v: &mut Editor, i: &Interface, mouse_info: MouseInfo) {
        if mouse_info.button != MouseButton::Left {
            v.set_behavior(Box::new(PanBehavior::new(i.viewport.clone(), mouse_info)));
            return;
        };

        v.begin_modification("Add point.");

        // We check if we have a point selected and are clicking on the beginning of another contour.
        // If that is the case we merge them
        if let (Some(fcidx), Some(fpidx)) = (v.contour_idx, v.point_idx) {
            // we've clicked a point?
            if let Some((lcidx, lpidx, _)) = clicked_point_or_handle(v, i, mouse_info.raw_position, None)
            {
                // we have the end of one contour active and clicked the start of another?
                let end_is_active =
                    get_contour_start_or_end(v, fcidx, fpidx) == Some(SelectPointInfo::End);
                let start_is_clicked =
                    get_contour_start_or_end(v, lcidx, lpidx) == Some(SelectPointInfo::Start);

                // make sure these contours are open
                let selected_open = is_contour_open!(v.get_active_layer_ref(), fcidx);
                let target_open = is_contour_open!(v.get_active_layer_ref(), lcidx);
                if end_is_active && start_is_clicked && selected_open && target_open {
                    // we're closing an open path
                    if fcidx == lcidx {
                        {
                            let layer = v.get_active_layer_mut();
                            let contour = get_contour_mut!(layer, fcidx);
                            
                            contour.set_closed();
                        }
                        
                        v.set_selected(fcidx, 0);
                        v.push_behavior(Box::new(MoveHandle::new(WhichHandle::A, mouse_info, true)));
                        return;
                    } else {
                        // TODO: Handle contourtype specific merging here. If the contour types don't match we resolve both to cubic types.
                        // If they do match we use a specific handler from the pen modes.
                        v.merge_contours(lcidx, fcidx)
                    }
                }
            }
        }

        // Next we check if our mouse is over an existing curve. If so we add a point to the curve.
        if let Some(info) = nearest_point_on_curve(v, i, mouse_info.position) {
            self.get_mode(v).subdivide_curve(v, info)
        }
        // If we've got the end of a contour selected we'll continue drawing that contour.
        else if can_add_point(v) {
            self.get_mode(v).add_point(v, mouse_info);
        } else {
            // Lastly if we get here we create a new contour.
            self.get_mode(v).new_contour(v, mouse_info)
        }

        // No matter how you move the point we want you to be able to manipulate it so we push the MoveHandle
        // vehavior onto the editor's behavior stack.
        v.push_behavior(Box::new(MoveHandle::new(WhichHandle::A, mouse_info, true)));
    }

    fn draw_nearest_point(&mut self, v: &Editor, i: &Interface, canvas: &mut Canvas) {
        if i.mouse_info.is_down {
            return;
        };
        let info = nearest_point_on_curve(v, i, i.mouse_info.position);

        if let Some(info) = info {
            self.get_mode(v).draw_nearest_point(i, canvas, info);
        }
    }

    fn draw_merge_preview(&self, v: &Editor, i: &Interface, canvas: &mut Canvas) {
        // we've got a point selected?
        if let (Some(c_idx), Some(p_idx)) = (v.contour_idx, v.point_idx) {
            // we've clicked a handle?
            if let Some((info_ci, info_pi, _)) =
                clicked_point_or_handle(v, i, i.mouse_info.raw_position, None)
            {
                // we have the end of one contour active and clicked the start of another?
                let end_is_active =
                    get_contour_start_or_end(v, c_idx, p_idx) == Some(SelectPointInfo::End);
                let start_is_clicked =
                    get_contour_start_or_end(v, info_ci, info_pi) == Some(SelectPointInfo::Start);

                // make sure these contours are open
                let selected_open = is_contour_open!(v.get_active_layer_ref(), c_idx);
                let target_open = is_contour_open!(v.get_active_layer_ref(), info_ci);
                if end_is_active && start_is_clicked && selected_open && target_open {
                    let point = get_contour!(v.get_active_layer_ref(), info_ci).get_point(p_idx).unwrap().clone();
                    draw_point(
                        &i.viewport,
                        point,
                        None,
                        true,
                        canvas,
                    );
                }
            }
        }
    }

    fn get_mode(&mut self, v: &Editor) -> &mut dyn PenMode {
        if can_add_point(v) {
            let (cidx, _) = v.selected_point().unwrap();
            return self.get_mode_for_contour(v, cidx);
        } else {
            return self.get_mode_by_type(self.mode.clone());
        }
    }

    fn get_mode_for_contour(&mut self, v: &Editor, cidx: usize) -> &mut dyn PenMode {
        let contour = &get_contour!(v.get_active_layer_ref(), cidx);
        
        return self.get_mode_by_type(contour.get_type());
    }

    fn get_mode_by_type(&mut self, kind: MFEKContourInnerType) -> &mut dyn PenMode {
        match kind {
            MFEKContourInnerType::Cubic => &mut self.cubic,
            MFEKContourInnerType::Quad => &mut self.quad,
            MFEKContourInnerType::Hyper => &mut self.hyper,
        }
    }
}

// quick utility function
fn can_add_point(v: &Editor) -> bool {
    if let Some((contour_idx, point_idx)) = v.selected_point() {
        let contour = &v.get_active_layer_ref().outline[contour_idx]; 

        if contour.is_closed() {
            return false
        }
        if point_idx == 0 || point_idx == contour.len() - 1 {
            return true
        }
        else {
            return false
        }
    } else {
        return false
    }
}

fn contour_type_to_string(mode: MFEKContourInnerType) -> usize {
    match mode {
        MFEKContourInnerType::Cubic => 0,
        MFEKContourInnerType::Quad => 1,
        MFEKContourInnerType::Hyper => 2,
    }
}