/*use glifparser::glif::{inner::MFEKContourInnerType, contour::MFEKContourCommon};

use self::cubic::CubicHandler;

use super::Editor;

pub mod cubic;
pub mod quad;

// ContourHandler holds any behaviors that are specific to a counter type (merging for instance) that exists outside
// the pen tool. When adding a new contour type/pen tool you will need to add the accompanying ContourHandler as well.
pub trait ContourHandler: std::fmt::Debug + Send {
    // Merge two contours together. The contours are garunteed to be the same ContourType by the time
    // this function is called within a PenMode.
    fn merge_contours(&self, v: &mut Editor, start_contour: usize, end_contour: usize);
}

impl Editor {
    pub fn get_contour_handler(&self, ci: usize) -> Box<dyn ContourHandler> {
        let kind = self.get_active_layer_ref().outline[ci].get_type();

        match kind {
            MFEKContourInnerType::Cubic => Box::new(CubicHandler{}),
            MFEKContourInnerType::Quad => Box::new(QuadHandler{}),
        }
    }
}*/
