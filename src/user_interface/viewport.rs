use crate::{HandleStyle, PointLabels, PreviewMode};

use super::Interface;

pub struct Viewport {
    pub winsize: (u32, u32),
    pub factor: f32,
    pub offset: (f32, f32),
    pub dpi: f64,
    pub point_labels: PointLabels,
    pub handle_style: HandleStyle,
    pub preview_mode: PreviewMode,
}

impl Default for Viewport {
    fn default() -> Self { 
        Viewport {
            winsize: (0, 0),
            factor: 1.,
            offset: (0., 0.),
            dpi: 1.,
            point_labels: PointLabels::None,
            preview_mode: PreviewMode::None,
            handle_style: HandleStyle::Handlebars,
        }
    }
}

impl Interface {  
    // this gets called by tools so it accepts &mut State
    pub fn update_viewport(&mut self, offset: Option<(f32, f32)>, scale: Option<f32>) {
        let offset = match offset {
            None => self.viewport.offset,
            Some(offset) => (self.viewport.offset.0 + offset.0, self.viewport.offset.1 + offset.1),
        };
        let scale = match scale {
            None => self.viewport.factor,
            Some(scale) => scale,
        };

        self.viewport.factor = scale;
        self.viewport.offset = offset;
    }
}