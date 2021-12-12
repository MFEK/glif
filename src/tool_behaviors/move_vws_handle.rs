use super::prelude::*;
use crate::tools::vws::util::*;

#[derive(Clone, Debug)]
pub struct MoveVWSHandle {
    mirror: bool,    // if true mirrors edits across the sides of the contour
    constrain: bool, // if true constrains the angle to the normal of the curve at the point
    all: bool,
    handle: WhichHandle,
    mouse_info: MouseInfo,
}

impl MoveVWSHandle {
    pub fn new(
        mirror: bool,
        constrain: bool,
        all: bool,
        handle: WhichHandle,
        mouse_info: MouseInfo,
    ) -> Self {
        MoveVWSHandle {
            mirror,
            constrain,
            all,
            handle,
            mouse_info,
        }
    }

    fn mouse_moved(&self, v: &mut Editor, _i: &Interface, mouse_info: MouseInfo) {
        if !v.is_modifying() {
            v.begin_modification("Move VWS handle.")
        }

        let (normal_offset, tangent_offset) =
            mouse_coords_to_handle_space(v, mouse_info, self.handle);
        // if shift is held down we scale all the points
        if self.all {
            set_all_vws_handles(v, self.handle, self.mirror, normal_offset)
        } else {
            set_vws_handle(
                v,
                self.handle,
                self.mirror,
                self.constrain,
                normal_offset,
                tangent_offset,
            )
        }
    }

    pub fn mouse_released(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        if !v.is_modifying() {
            return; // just a click
        }
        if mouse_info.button == self.mouse_info.button {
            v.end_modification();
            v.pop_behavior();
        }
    }
}

#[rustfmt::skip]
impl ToolBehavior for MoveVWSHandle {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        if let EditorEvent::MouseEvent { mouse_info, event_type } = event {
            match event_type {
                MouseEventType::Released => self.mouse_released(v, i, mouse_info),
                MouseEventType::Moved => self.mouse_moved(v, i, mouse_info),
                _ => (),
            }
        }
    }
}
