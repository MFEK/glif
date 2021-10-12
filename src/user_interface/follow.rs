pub use crate::user_interface::MouseInfo;
use derive_more::Display;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
/// Point following behavior when using the select tool
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
                } else {
                    Follow::No
                }
            }
            MouseInfo {
                button: MouseButton::Right,
                ..
            } => Follow::Mirror,
            _ => Follow::QuadOpposite,
        }
    }
}
