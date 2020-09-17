use derive_more::Display;
use glifparser::WhichHandle;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
/// Point following behavior when using the arrow tool
pub enum Follow {
    // Other point will take mirror action of current point.
    Mirror,
    // Other point will be forced into a line with the current point as midpoint.
    ForceLine,
    // For a quadratic curve, the "other side" of the curve (in reality, the control point on an
    // adjacent curve), should follow too.
    QuadOpposite,
    // Other point will remain in fixed position.
    No,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToolData {
    pub contour: Option<usize>,   // index into Outline
    pub cur_point: Option<usize>, // index into Contour
    pub handle: WhichHandle,      // if handle of cur_point selected, not point
    pub follow: Follow,           // determined currently by MouseButton
}

impl ToolData {
    fn new() -> Self {
        ToolData {
            contour: None,
            cur_point: None,
            handle: WhichHandle::Neither,
            follow: Follow::Mirror,
        }
    }
}

use crate::events::MouseMeta;
use crate::winit::event::MouseButton;
impl From<MouseMeta> for Follow {
    fn from(m: MouseMeta) -> Follow {
        match m {
            MouseMeta {
                button: MouseButton::Left,
                modifiers,
            } => {
                if modifiers.ctrl() {
                    Follow::ForceLine
                } else {
                    Follow::Mirror
                }
            }
            MouseMeta {
                button: MouseButton::Right,
                ..
            } => Follow::No,
            _ => Follow::QuadOpposite,
        }
    }
}

use std::cell::RefCell;
thread_local!(pub static TOOL_DATA: RefCell<ToolData> = RefCell::new(ToolData::new()));
