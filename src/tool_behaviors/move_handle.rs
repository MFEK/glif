use super::prelude::*;
use crate::get_point_mut;
pub use crate::user_interface::follow::Follow;
use glifparser::PointData;
use glifparser::glif::contour::MFEKContourCommon;
use glifparser::glif::point::MFEKPointCommon;
use MFEKmath::polar::PolarCoordinates;

#[derive(Clone, Debug)]
pub struct MoveHandle {
    // we hold on to a clone of the mouse info when the behavior gets put on the stack
    // on mouse released we check for the same button found here and if we find it we pop ourselves
    // this is a common convention among toolbehaviors and allows the behavior to be independent of bindings
    mouse_info: MouseInfo,

    // the index of the point that has the handle we're moving
    wh: WhichHandle,

    // this handle did not previously exist but is being drawn by user for the first time
    creating: bool,
}

// Event handlers
impl MoveHandle {
    pub fn new(wh: WhichHandle, mouse_info: MouseInfo, creating: bool) -> Self {
        Self {
            wh,
            mouse_info,
            creating,
        }
    }

    pub fn mouse_moved(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        if !v.is_modifying() {
            v.begin_modification("Move handle.", false);
        }

        self.mouse_info.modifiers = mouse_info.modifiers;

        let (x, y) = mouse_info.position;

        let (vci, vpi) = (v.contour_idx.unwrap(), v.point_idx.unwrap());

        {
            let layer = v.get_active_layer_mut();
            let point = get_point_mut!(layer, vci, vpi).unwrap();
            let handle = point.get_handle(self.wh).unwrap();

            // Current x, current y
            let (cx, cy) = match handle {
                Handle::At(cx, cy) => (cx, cy),
                Handle::Colocated => (point.x(), point.y()),
            };

            // Difference in x, difference in y
            let (dx, dy) = (cx - x, cy - y);

            let mut follow = if self.creating {
                Follow::Mirror
            } else {
                Follow::from(self.mouse_info)
            };

            if point.get_smooth() == Some(true) {
                // Follow::Mirror breaks smoothness
                follow = Follow::ForceLine;
            }

            if point.get_handle(self.wh.opposite()).is_some_and(|wh| wh == Handle::Colocated) && follow == Follow::ForceLine {
                // If the opposite handle is colocated, force line crashes due to unwrapping a None value.
                follow = Follow::No;
            }
            match self.wh {
                WhichHandle::A | WhichHandle::B => {
                    point.set_handle(self.wh, Handle::At(x, y));
                    match follow {
                        Follow::Mirror => self.mirror(point, (dx, dy)),
                        Follow::ForceLine => self.force_line(point),
                        Follow::No => (),
                    }
                }
                WhichHandle::Neither => unreachable!("Should've been matched above?!"),
            }

            //get_contour_mut!(layer, vci).refigure_point_types();
        }
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
    fn mirror<PD: PointData>(&self, point: &mut dyn MFEKPointCommon<PD>, (dx, dy): (f32, f32)) {
        if let Some(Handle::At(hx, hy)) = point.get_handle(self.wh.opposite()) {
            point.set_handle(self.wh.opposite(), Handle::At(hx + dx, hy + dy));
        } else if let Some(Handle::Colocated) = point.get_handle(self.wh.opposite()) {
            point.set_handle(self.wh.opposite(), Handle::At(point.x() + dx, point.y() + dy));
        }
    }

    fn force_line<PD: PointData>(&self, mut point: &mut dyn MFEKPointCommon<PD>) {
        let (r, _) = point.polar(self.wh.opposite());
        let (_, theta) = point.polar(self.wh);
        match point.get_handle(self.wh.opposite()).unwrap() {
            Handle::At(..) => point.set_polar(self.wh.opposite(), (r, theta.to_degrees())),
            Handle::Colocated => unreachable!("Force line is nonsensical when the opposite handle is colocated")
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
