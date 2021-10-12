use super::prelude::*;
pub use crate::user_interface::follow::Follow;

#[derive(Clone)]
pub struct MoveHandle {
    // we hold on to a clone of the mouse info when the behavior gets put on the stack
    // on mouse released we check for the same button found here and if we find it we pop ourselves
    // this is a common process among toolbehaviors and allows the behavior to be independent of bindings
    mouse_info: MouseInfo,
    follow: Follow,

    // the index of the point that has the handle we're moving
    wh: WhichHandle,
}

impl MoveHandle {
    pub fn new(wh: WhichHandle, follow: Follow, mouse_info: MouseInfo) -> Self {
        Self {
            wh,
            follow,
            mouse_info,
        }
    }

    pub fn mouse_moved(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        if !v.is_modifying() {
            v.begin_modification("Move handle.");
        }

        let x = calc_x(mouse_info.position.0 as f32);
        let y = calc_y(mouse_info.position.1 as f32);

        let (vci, vpi) = (v.contour_idx.unwrap(), v.point_idx.unwrap());

        v.with_active_layer_mut(|layer| {
            let handle = match self.wh {
                WhichHandle::A => get_point!(layer, vci, vpi).a,
                WhichHandle::B => get_point!(layer, vci, vpi).b,
                WhichHandle::Neither => unreachable!("Should've been matched by above?!"),
            };

            // Current x, current y
            let (cx, cy) = match handle {
                Handle::At(cx, cy) => (cx, cy),
                _ => {
                    let point_pos = (get_point!(layer, vci, vpi).x, get_point!(layer, vci, vpi).y);
                    // we are going to initialize the other handle if follow is
                    // set to mirror
                    if self.follow == Follow::Mirror {
                        let initialize_pos = Handle::At(point_pos.0, point_pos.1);
                        match self.wh {
                            WhichHandle::A => get_point!(layer, vci, vpi).b = initialize_pos,
                            WhichHandle::B => get_point!(layer, vci, vpi).a = initialize_pos,
                            WhichHandle::Neither => {
                                unreachable!("Should've been matched by above?!")
                            }
                        }
                    }
                    point_pos
                }
            };

            // Difference in x, difference in y
            let (dx, dy) = (cx - x, cy - y);

            // If Follow::Mirror (left mouse button), other control point (handle) will do mirror
            // image action of currently selected control point. Perhaps pivoting around central
            // point is better?
            macro_rules! move_mirror {
                ($cur:ident, $mirror:ident) => {
                    get_point!(layer, vci, vpi).$cur = Handle::At(x, y);
                    let h = get_point!(layer, vci, vpi).$mirror;
                    match h {
                        Handle::At(hx, hy) => {
                            if self.follow == Follow::Mirror {
                                get_point!(layer, vci, vpi).$mirror = Handle::At(hx + dx, hy + dy);
                            } else if self.follow == Follow::ForceLine {
                                let (px, py) =
                                    (get_point!(layer, vci, vpi).x, get_point!(layer, vci, vpi).y);
                                let (dx, dy) = (px - x, py - y);

                                get_point!(layer, vci, vpi).$mirror = Handle::At(px + dx, py + dy);
                            }
                        }
                        Handle::Colocated => (),
                    }
                };
            }

            match self.wh {
                WhichHandle::A => {
                    move_mirror!(a, b);
                }
                WhichHandle::B => {
                    move_mirror!(b, a);
                }
                WhichHandle::Neither => unreachable!("Should've been matched by above?!"),
            }
        });
    }

    pub fn mouse_released(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        if mouse_info.button == self.mouse_info.button {
            v.end_modification();
            v.pop_behavior();
        }
    }
}

impl ToolBehavior for MoveHandle {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent {
                event_type,
                mouse_info,
            } => match event_type {
                MouseEventType::Released => self.mouse_released(v, i, mouse_info),
                MouseEventType::Moved => self.mouse_moved(v, i, mouse_info),
                _ => {}
            },
            _ => {}
        }
    }
}
