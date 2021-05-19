use MFEKmath::{Piecewise, VWSSettings, variable_width_stroke};
use glifparser::glif::{ContourOperations, InterpolationType, MFEKContour, VWSHandle};

use crate::contour_operations;

use super::Editor;


impl Editor {
    pub fn mark_preview_dirty(&mut self)
    {
        self.preview_dirty = true;

    }

    pub fn rebuild(&mut self) {
        if !self.preview_dirty {return};

        if self.glyph.as_ref().unwrap().layers[0].operation.is_some() {
            self.glyph.as_mut().unwrap().layers[0].operation = None;
        }

        //self.fix_contour_ops();
        let mut preview_layers = Vec::new();
        for layer in &self.glyph.as_ref().unwrap().layers {
            let mut preview_outline = Vec::new();

            for (idx, glif_contour) in layer.outline.iter().enumerate() {
                if glif_contour.inner.len() < 2 { preview_outline.push(glif_contour.clone()); continue; }

                let build_result = contour_operations::build(glif_contour);

                for new_contour in build_result {
                    preview_outline.push(new_contour);
                }
            }

            let mut new_layer = layer.clone();
            new_layer.outline = preview_outline;
            preview_layers.push(new_layer);
        }

        self.preview = Some(self.glyph.as_ref().unwrap().clone());
        self.preview.as_mut().unwrap().layers = preview_layers;
        self.preview_dirty = false;
    }

    /* 
    // this call checks if the contour ops are in tact and have information for all of it's points
    // before we build the previews
    pub fn fix_contour_ops(&mut self)
    {
        for layer in &mut self.glyph.as_mut().unwrap().layers {
            for (idx, glif_contour) in &mut layer.outline.iter().enumerate() {
                match layer.contour_ops.get(&idx) {
                    Some(contour_op) => {
                        match contour_op {
                            ContourOp::VariableWidthStroke { contour } => {
                                let mut new_contour = contour.clone();
                                while glif_contour.inner.len() + 1 > new_contour.handles.len() {
                                    new_contour.handles.push(VWSHandle{
                                        left_offset: contour.handles.last().unwrap().left_offset,
                                        right_offset: contour.handles.last().unwrap().right_offset,
                                        interpolation: InterpolationType::Linear,
                                        tangent_offset: 0.,
                                    })
                                }

                                while new_contour.handles.len() > glif_contour.inner.len() + 1 {
                                    new_contour.handles.pop();
                                }

                                layer.contour_ops.insert(idx, ContourOp::VariableWidthStroke { contour: new_contour} );
                            }
                        }
                    }
                    None => {
                    }
                }
            }
        }
    } */
}
