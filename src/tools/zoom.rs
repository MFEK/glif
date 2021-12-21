use sdl2::mouse::MouseButton;

use crate::tool_behaviors::zoom_scroll::ZoomScroll;
use crate::user_interface::Interface;
use crate::{constants::SCALE_FACTOR, editor::Editor};

// Pan
use super::{prelude::*, EditorEvent, MouseEventType, Tool};

#[derive(Clone, Debug)]
pub struct Zoom {}

#[rustfmt::skip]
impl Tool for Zoom {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent { mouse_info, event_type } => match event_type {
                MouseEventType::Released => self.mouse_released(i, mouse_info),
                _ => (),
            }
            EditorEvent::ScrollEvent { .. } => ZoomScroll::default().event(v, i, event),
            _ => {},
        }
    }
}

impl Zoom {
    pub fn new() -> Self {
        Self {}
    }

    fn mouse_released(&self, i: &mut Interface, mouse_info: MouseInfo) {
        let scale = match mouse_info.button {
            MouseButton::Left => zoom_in_factor(i),
            MouseButton::Right => zoom_out_factor(i),
            _ => return,
        };
        i.update_viewport(None, Some(scale));
    }
}

// TODO: Move these both to glifrenderer.rlib ⇒ viewport.rs
pub fn zoom_in_factor(i: &mut Interface) -> f32 {
    i.viewport.factor + SCALE_FACTOR
}

pub fn zoom_out_factor(i: &mut Interface) -> f32 {
    let mut scale = i.viewport.factor;
    if scale >= 0.10 {
        scale += -SCALE_FACTOR;
    }
    scale
}
