use crate::{HandleStyle, PointLabels, PreviewMode};

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