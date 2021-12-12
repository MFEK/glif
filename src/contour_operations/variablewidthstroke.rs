use glifparser::{
    glif::{self, MFEKContour, MFEKOutline, VWSHandle},
    VWSContour,
};
use MFEKmath::{variable_width_stroke, Piecewise, VWSSettings};

use super::ContourOperation;
use crate::util::MFEKGlifPointData;

impl ContourOperation for VWSContour {
    fn build(&self, contour: &MFEKContour<MFEKGlifPointData>) -> MFEKOutline<MFEKGlifPointData> {
        let contour_pw = Piecewise::from(&contour.inner);

        let settings = VWSSettings::<MFEKGlifPointData> {
            cap_custom_start: None,
            cap_custom_end: None,
        };

        let vws_output = variable_width_stroke(&contour_pw, self, &settings);

        let mut output: MFEKOutline<MFEKGlifPointData> = Vec::new();
        for contour in vws_output.segs {
            output.push(contour.to_contour().into());
        }

        output
    }

    fn sub(&self, _contour: &MFEKContour<MFEKGlifPointData>, begin: usize, end: usize) -> Self {
        let temp_handles = self.handles.split_at(begin);
        let (final_handles, _) = temp_handles.1.split_at(end + 1 - begin);
        VWSContour {
            handles: final_handles.into(),
            join_type: self.join_type,
            cap_start_type: self.cap_start_type,
            cap_end_type: self.cap_end_type,
            remove_internal: self.remove_internal,
            remove_external: self.remove_external,
        }
    }

    fn append(
        &self,
        _contour: &MFEKContour<MFEKGlifPointData>,
        append: &MFEKContour<MFEKGlifPointData>,
    ) -> Self {
        let mut temp_handles = self.handles.clone();

        match append.operation.clone() {
            Some(glifparser::glif::ContourOperations::VariableWidthStroke { mut data }) => {
                temp_handles.append(&mut data.handles)
            }
            Some(_) => {
                for _idx in 0..append.inner.len() {
                    let last_handle = *(temp_handles.last().unwrap_or(&VWSHandle {
                        left_offset: 10.,
                        right_offset: 10.,
                        tangent_offset: 0.,
                        interpolation: glif::InterpolationType::Linear,
                    }));
                    temp_handles.push(last_handle);
                }
            }
            None => {
                for _idx in 0..append.inner.len() {
                    let last_handle = *(temp_handles.last().unwrap_or(&VWSHandle {
                        left_offset: 10.,
                        right_offset: 10.,
                        tangent_offset: 0.,
                        interpolation: glif::InterpolationType::Linear,
                    }));
                    temp_handles.push(last_handle);
                }
            }
        }

        VWSContour {
            handles: temp_handles,
            join_type: self.join_type,
            cap_start_type: self.cap_start_type,
            cap_end_type: self.cap_end_type,
            remove_internal: self.remove_internal,
            remove_external: self.remove_external,
        }
    }

    fn insert(&self, _contour: &MFEKContour<MFEKGlifPointData>, point_idx: usize) -> Self {
        let mut temp_handles = self.handles.clone();
        temp_handles.insert(
            point_idx,
            VWSHandle {
                left_offset: temp_handles[point_idx].left_offset,
                right_offset: temp_handles[point_idx].right_offset,
                tangent_offset: temp_handles[point_idx].tangent_offset,
                interpolation: temp_handles[point_idx].interpolation,
            },
        );

        VWSContour {
            handles: temp_handles,
            join_type: self.join_type,
            cap_start_type: self.cap_start_type,
            cap_end_type: self.cap_end_type,
            remove_internal: self.remove_internal,
            remove_external: self.remove_external,
        }
    }
}
