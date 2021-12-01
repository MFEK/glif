use super::prelude::*;

use crate::editor::Editor;
use crate::tool_behaviors::ToolBehavior;
use crate::tool_behaviors::pan::PanBehavior;
use crate::user_interface::Interface;

// Pan is a good example of a simple tool. It holds only a little bit of state, an optional position.
// If the optional position is set the tool will calculate the delta between that position and the current
// mouse position and set the camera accordingly.
#[derive(Clone, Debug)]
pub struct Pan {
    #[allow(unused)]
    last_position: Option<(f32, f32)>,
}

// We implement Tool for our tool. Here you can route events to functions or implement logic directly in the
// match statement.
impl Tool for Pan {
    fn event(&mut self, v: &mut Editor, interface: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent {
                event_type,
                mouse_info,
            } => match event_type {
                MouseEventType::Pressed => v.push_behavior(Box::new(PanBehavior::new(
                    interface.viewport.clone(),
                    mouse_info,
                ))),
                _ => {}
            },
            EditorEvent::ToolCommand { .. } => {
                let mut behavior = Box::new(PanBehavior::new(
                    interface.viewport.clone(),
                    MouseInfo::default(),
                ));
                behavior.event(v, interface, event);
            },
        }
    }
}

// Here you can implement behaviors for events.
impl Pan {
    pub fn new() -> Self {
        Self {
            last_position: None,
        }
    }
}
