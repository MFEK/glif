mod prelude;
pub mod pan;
pub mod move_handle;
pub mod move_point;
pub mod selection_box;
pub mod rotate_selection;
pub mod add_guideline;
pub mod move_guideline;
pub mod move_image;
pub mod rotate_image;
pub mod move_vws_handle;

use self::prelude::*;
use dyn_clone::DynClone;

/// A ToolBehavior is a subset of Tool. These are pushed onto the editor's tool_behaviors stack.
/// If there is a tool on the stack the top ToolBehavior will intercept events. This allows you to
/// encapsulate behaviors like panning, and reuse them. It also allows you to break large tools like Select
/// up into more manageable pieces.
pub trait ToolBehavior: DynClone {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent);

    // Not every behavior draws so we provide an empty default implementation.
    fn draw(&self, _v: &Editor, _i: &Interface, _canvas: &mut Canvas) {}
}

