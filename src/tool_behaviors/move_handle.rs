use super::prelude::*;
pub use crate::user_interface::follow::Follow;
use glifparser::outline::RefigurePointTypes as _;
use MFEKmath::polar::PolarCoordinates as _;

#[derive(Clone, Debug)]
pub struct MoveHandle {
    // we hold on to a clone of the mouse info when the behavior gets put on the stack
    // on mouse released we check for the same button found here and if we find it we pop ourselves
    // this is a common process among toolbehaviors and allows the behavior to be independent of bindings
    mouse_info: MouseInfo,

    // the index of the point that has the handle we're moving
    wh: WhichHandle,

    // this handle did not previously exist but is being drawn by user for the first time
    creating: bool,

    warned_force_line: bool,
}

// Event handlers
impl MoveHandle {
    pub fn new(wh: WhichHandle, mouse_info: MouseInfo, creating: bool) -> Self {
        Self {
            wh,
            mouse_info,
            creating,
            warned_force_line: false,
        }
    }

    pub fn mouse_moved(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        if !v.is_modifying() {
            v.begin_modification("Move handle.");
        }

        self.mouse_info.modifiers = mouse_info.modifiers;

        let x = calc_x(mouse_info.position.0 as f32);
        let y = calc_y(mouse_info.position.1 as f32);

        let (vci, vpi) = (v.contour_idx.unwrap(), v.point_idx.unwrap());

        v.with_active_layer_mut(|layer| {
            let handle = match self.wh {
                WhichHandle::A => get_point!(layer, vci, vpi).a,
                WhichHandle::B => get_point!(layer, vci, vpi).b,
                WhichHandle::Neither => {
                    panic!("MoveHandle cannot be created with a WhichHandle::Neither!")
                }
            };

            // Current x, current y
            let (cx, cy) = match handle {
                Handle::At(cx, cy) => (cx, cy),
                Handle::Colocated => (get_point!(layer, vci, vpi).x, get_point!(layer, vci, vpi).y),
            };

            // Difference in x, difference in y
            let (dx, dy) = (cx - x, cy - y);

            let follow = if self.creating {
                Follow::Mirror
            } else {
                Follow::from(self.mouse_info)
            };

            match self.wh {
                WhichHandle::A | WhichHandle::B => {
                    get_point!(layer, vci, vpi).set_handle(self.wh, Handle::At(x, y));
                    match follow {
                        Follow::Mirror => self.mirror(&mut get_point!(layer, vci, vpi), (dx, dy)),
                        Follow::ForceLine => self.force_line(&mut get_point!(layer, vci, vpi)),
                        Follow::No => (),
                    }
                }
                WhichHandle::Neither => unreachable!("Should've been matched above?!"),
            }

            get_contour_mut!(layer, vci).refigure_point_types();
        });
    }

    pub fn mouse_released(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        if mouse_info.button == self.mouse_info.button {
            v.end_modification();
            v.pop_behavior();
        }
    }
}

// Implementations of Follow::* behaviors
impl MoveHandle {
    fn mirror(&self, point: &mut Point<MFEKGlifPointData>, (dx, dy): (f32, f32)) {
        let (hx, hy) = match point.handle(self.wh.opposite()) {
            Handle::At(hx, hy) => (hx, hy),
            Handle::Colocated => {
                // Initialize control point currently marked as being colocated
                point.set_handle(self.wh.opposite(), Handle::At(point.x, point.y));
                (point.x, point.y)
            }
        };
        point.set_handle(self.wh.opposite(), Handle::At(hx + dx, hy + dy));
    }

    fn force_line(&mut self, point: &mut Point<MFEKGlifPointData>) {
        let (r, _) = point.polar(self.wh.opposite());
        let (_, theta) = point.polar(self.wh);
        match point.handle(self.wh.opposite()) {
            Handle::At(..) => point.set_polar(self.wh.opposite(), (r, theta.to_degrees())),
            Handle::Colocated => {
                if !self.warned_force_line {
                    log::warn!(
                        "Makes no sense to force line when opposite handle is colocated, ignoring"
                    );
                }
                self.warned_force_line = true;
            }
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
