pub use crate::user_interface::MouseInfo;
use derive_more::Display;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
/// Point following behavior when using the select tool
pub enum Follow {
    // If Follow::Mirror (left mouse button), other control point (handle) will do mirror
    // image action of currently selected control point. Perhaps pivoting around central
    // point is better?
    /// Other point will take mirror action of current point.
    Mirror,
    /// Other point will be forced into a line with the current point as midpoint.
    ForceLine,
    /// Other point will remain in fixed position.
    No,
}

use sdl2::mouse::MouseButton;
impl From<MouseInfo> for Follow {
    fn from(m: MouseInfo) -> Follow {
        match m {
            MouseInfo {
                button: MouseButton::Left,
                modifiers,
                ..
            } => {
                if modifiers.ctrl {
                    Follow::ForceLine
                } else if modifiers.alt {
                    Follow::Mirror
                } else {
                    Follow::No
                }
            }
            MouseInfo {
                button: MouseButton::Right,
                ..
            } => Follow::Mirror,
            _ => Follow::No,
        }
    }
}
