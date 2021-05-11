use MFEKmath::{Piecewise, VWSSettings, variable_width_stroke};
use glifparser::glif::{ContourOp, InterpolationType, MFEKContour, VWSHandle};
use super::Editor;


impl Editor {
    pub fn rebuild(&mut self)
    {
        if self.glyph.as_ref().unwrap().layers[0].operation.is_some() {
            self.glyph.as_mut().unwrap().layers[0].operation = None;
        }

        self.fix_contour_ops();
        let mut preview_layers = Vec::new();
        for layer in &self.glyph.as_ref().unwrap().layers {
            let mut preview_outline = Vec::new();

            for (idx, glif_contour) in layer.outline.iter().enumerate() {
                if glif_contour.inner.len() < 2 { preview_outline.push(glif_contour.clone()); continue; }

                match layer.contour_ops.get(&idx) {
                    Some(contour_op) => {
                        match contour_op {
                            ContourOp::VariableWidthStroke { contour } => {
                                let contour_pw = Piecewise::from(&glif_contour.inner);
                        
                                let settings = VWSSettings {
                                    cap_custom_start: None,
                                    cap_custom_end: None,
                                };
                        
                                let vws_output = variable_width_stroke(&contour_pw, contour, &settings);
                        
                                for contour in vws_output.segs {
                                    preview_outline.push(contour.to_contour().into());
                                }
                            }
                        }
                    }
                    None => {
                        preview_outline.push(glif_contour.clone());
                    }
                }
            }

            let mut new_layer = layer.clone();
            new_layer.outline = preview_outline;
            preview_layers.push(new_layer);
        }

        self.preview = Some(self.glyph.as_ref().unwrap().clone());
        self.preview.as_mut().unwrap().layers = preview_layers;
    }

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
    }
}