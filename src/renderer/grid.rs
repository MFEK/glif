use skulpin::skia_safe::Color4f;
use skulpin::skia_safe::Paint;
use skulpin::skia_safe::Path;
use skulpin::skia_safe::PaintStyle;

use crate::renderer::constants::GUIDELINE_THICKNESS;
use crate::user_interface::viewport::Viewport;
use crate::user_interface::grid::Grid;
use crate::renderer::Canvas;

pub fn draw_grid(canvas: &mut Canvas, grid: &Grid, viewport: &Viewport) { 
    let mut grid_path = Path::new();

    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_stroke_width(GUIDELINE_THICKNESS * (1. / viewport.factor));
    paint.set_style(PaintStyle::Stroke);
    paint.set_color4f(Color4f::new(0., 0., 0., 0.2), None);

    let scaled_left_offset = viewport.offset.0 / viewport.factor;
    let scaled_top_offset = viewport.offset.1 / viewport.factor;

    let whole_left_offset = (grid.offset + scaled_left_offset) / grid.spacing;
    let fractional_left_offset = whole_left_offset - whole_left_offset.floor();
    let units_from_left = fractional_left_offset * grid.spacing;
    
    let total_vertical = f32::floor(viewport.winsize.0 as f32 / viewport.factor / grid.spacing) as usize;
    for i in 0..total_vertical {
        grid_path.move_to((units_from_left - scaled_left_offset + i as f32 * grid.spacing, -scaled_top_offset));
        grid_path.line_to((units_from_left - scaled_left_offset + i as f32 * grid.spacing, -scaled_top_offset + viewport.winsize.1 as f32 / viewport.factor));
    }

    let whole_top_offset = (grid.offset + scaled_top_offset) / grid.spacing;
    let fractional_top_offset = whole_top_offset - whole_top_offset.floor();
    let units_from_top = fractional_top_offset * grid.spacing;

    for i in 0..1000 {
        grid_path.move_to((-scaled_left_offset, units_from_top - scaled_top_offset + i as f32 * grid.spacing));
        grid_path.line_to((-scaled_left_offset + viewport.winsize.0 as f32 / viewport.factor, units_from_top - scaled_top_offset + i as f32 * grid.spacing));
    }

    canvas.draw_path(&grid_path, &paint);
}