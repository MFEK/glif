use crate::get_point_mut;

use super::prelude::*;
use glifparser::glif::{mfek::contour::MFEKContourCommon};
#[derive(Clone, Debug)]
pub struct MovePoint {
    // we hold on to a clone of the mouse info when the behavior gets put on the stack
    // on mouse released we check for the same button found here and if we find it we pop ourselves
    // this is a common process among toolbehaviors and allows the behavior to be independent of bindings
    mouse_info: MouseInfo,

    // should we move all selected points?
    move_selected: bool,
}

impl MovePoint {
    pub fn new(move_selected: bool, mouse_info: MouseInfo) -> Self {
        Self {
            mouse_info,
            move_selected,
        }
    }

    pub fn mouse_moved(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        let x = mouse_info.position.0 as f32;
        let y = mouse_info.position.1 as f32;

        let (vci, vpi) = v.selected_point().unwrap();
        if !v.is_modifying() {
            v.begin_modification("Move point.", false);
        }

        let layer = v.get_active_layer_mut();
        let point = get_point_mut!(layer, vci, vpi).unwrap();
        let previous_position = point.get_position();

        point.set_position(x, y);
        let selected = v.selected.clone();
        {
            let layer = v.get_active_layer_mut();
            if self.move_selected {
                for (ci, pi) in &selected {
                    if *ci == vci && *pi == vpi { continue; }
                    let (ci, pi) = (*ci, *pi);
                    let point = get_point_mut!(layer, ci, pi).unwrap();
                    let offset_x = x - previous_position.0;
                    let offset_y = y - previous_position.1;
                    point.set_position(point.x() + offset_x, point.y() + offset_y);
                }
            }
        }
    }

    pub fn mouse_released(&mut self, v: &mut Editor, i: &mut Interface, mouse_info: MouseInfo) {
        if mouse_info.button == self.mouse_info.button {
            let (vci, vpi) = (v.contour_idx.unwrap(), v.point_idx.unwrap());

            // we are going to check if we're dropping this point onto another and if this is the end, and that the
            // start or vice versa if so we're going to merge but first we have to check we're dragging a point
            if v.is_modifying() {
                // are we overlapping a point?
                if let Some((ci, pi, WhichHandle::Neither)) =
                    clicked_point_or_handle(v, i, mouse_info.raw_position, Some((vci, vpi)))
                {
                    // if that point the start or end of it's contour?
                    if let Some(info) = get_contour_start_or_end(v, vci, vpi) {
                        // is our current point the start or end of it's contour?
                        if let Some(target_info) = get_contour_start_or_end(v, ci, pi) {
                            let layer = v.get_active_layer_ref();

                            // do we have two starts or two ends?
                            if get_contour!(layer, vci).is_open() && get_contour!(layer, ci).is_open() && target_info != info
                            {
                                let start = if info == SelectPointInfo::Start {
                                    vci
                                } else {
                                    ci
                                };
                                let end = if info == SelectPointInfo::End {
                                    vci
                                } else {
                                    ci
                                };

                                if vci == ci {
                                    {
                                        let layer = v.get_active_layer_mut();
                                        let contour = get_contour_mut!(layer, vci);
                                        
                                        contour.delete(contour.len() - 1);
                                        contour.set_closed();
                                        v.set_selected(vci, 0);
                                    }
                                } else {
                                    v.merge_contours(start, end);
                                }
                            }
                        }
                    }
                } 
                v.end_modification();
            }

            v.pop_behavior();
        }
    }
}

impl ToolBehavior for MovePoint {
    #[rustfmt::skip]
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        if let EditorEvent::MouseEvent { mouse_info, event_type } = event {
            match event_type {
                MouseEventType::Released => self.mouse_released(v, i, mouse_info),
                MouseEventType::Moved => self.mouse_moved(v, i, mouse_info),
                _ => (),
            }
        }
    }

    // We draw a preview to show if the point will be merged with another when you let go of the mouse button.
    fn draw(&mut self, v: &Editor, i: &Interface, canvas: &Canvas) {
        // This draws a preview to show if we're overlapping a point we can merge with or not.
        // Note that all tool draw events draw over the glyph view.
        if v.contour_idx.is_none() || v.point_idx.is_none() {
            return;
        }

        let (vci, vpi) = (v.contour_idx.unwrap(), v.point_idx.unwrap());

        // are we overlapping a point?
        if let Some((ci, pi, WhichHandle::Neither)) =
            clicked_point_or_handle(v, i, i.mouse_info.raw_position, Some((vci, vpi)))
        {
            // if that point the start or end of it's contour?
            if let Some(info) = get_contour_start_or_end(v, vci, vpi) {
                // is our current point the start or end of it's contour?
                if let Some(target_info) = get_contour_start_or_end(v, ci, pi) {
                    let layer = v.get_active_layer_ref();

                    // do we have two starts or two ends?
                    if get_contour!(layer, vci).is_open() && get_contour!(layer, ci).is_open() && target_info != info
                    {
                        let merge = get_contour!(v.get_active_layer_ref(), ci).get_point(pi).unwrap();
                        draw_point(
                            &i.viewport,
                            merge,
                            None,
                            true,
                            canvas,
                        );
                    }
                }
            }
        }
    }
}
