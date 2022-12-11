pub mod cubic;
pub mod quad;
use MFEKmath::skia_safe::Canvas;
use dyn_clone::DynClone;
use crate::{editor::{Editor, util::HoveredPointInfo}, user_interface::{Interface, MouseInfo}};

pub trait PenMode: DynClone + std::fmt::Debug {
    // No selection starting to draw a new contour.
    fn new_contour(&self, v: &mut Editor, mouse_info: MouseInfo);

    // The start or end of a contour is selected and we're adding to it.
    fn add_point(&self, v: &mut Editor, mouse_info: MouseInfo);
    
    // If a contour type does not support subdivision the way that cubic or quadratic does then you can safely
    // stub these functions out.
    fn subdivide_curve(&self, v: &mut Editor, info: HoveredPointInfo);
    fn draw_nearest_point(&self, i: &Interface, canvas: &mut Canvas, info: HoveredPointInfo);
}