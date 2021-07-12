use skulpin::skia_safe::Color4f;
use skulpin::skia_safe::Paint;
use skulpin::skia_safe::Path;
use skulpin::skia_safe::PaintStyle;

use crate::renderer::constants::GUIDELINE_THICKNESS;
use crate::user_interface::viewport::Viewport;
use crate::user_interface::grid::Grid;
use crate::renderer::Canvas;
use crate::renderer::points::calc::*;

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
    
    let total_vertical = f32::floor(viewport.winsize.0 as f32 / viewport.factor / grid.spacing) as i32;
    for i in 0..total_vertical {
        grid_path.move_to((calc_x(units_from_left - scaled_left_offset + i as f32 * grid.spacing), -scaled_top_offset));
        grid_path.line_to((calc_x(units_from_left - scaled_left_offset + i as f32 * grid.spacing), -scaled_top_offset + viewport.winsize.1 as f32 / viewport.factor));
    }

    let whole_top_offset = (grid.offset - scaled_top_offset) / grid.spacing;
    let fractional_top_offset = whole_top_offset - whole_top_offset.floor();
    let units_from_top = fractional_top_offset * grid.spacing;

    let total_horizontal = f32::floor(viewport.winsize.1 as f32 / viewport.factor / grid.spacing) as i32;
    for i in -total_horizontal..total_horizontal {
        grid_path.move_to((calc_x(-scaled_left_offset), calc_y(units_from_top) - scaled_top_offset + i as f32 * grid.spacing));
        grid_path.line_to((calc_x(-scaled_left_offset + viewport.winsize.0 as f32 / viewport.factor), calc_y(units_from_top) - scaled_top_offset + i as f32 * grid.spacing));
    }

    if let Some(slope) = grid.slope {
        if slope == 0. { return };
        let slope_max = f32::max(slope, 1.);

        let viewx = viewport.winsize.0 as f32 /viewport.factor;

        let spacing = grid.spacing * slope_max;
        let extra = -total_horizontal + (-total_horizontal as f32 * slope) as i32;
        let offset = ((grid.offset + scaled_top_offset + scaled_left_offset * slope) / spacing).fract() * spacing;

        for i in extra..-extra {
            grid_path.move_to((calc_x(-scaled_left_offset), calc_y(spacing * i as f32 - offset + scaled_top_offset)));
            grid_path.line_to((
                calc_x(viewx - scaled_left_offset),
                calc_y(slope*(viewx) + spacing * i as f32 - offset + scaled_top_offset)
            ));
        }
        /*
        let whole_italic_offset = (grid.offset + scaled_left_offset - scaled_top_offset * slope) / grid.spacing;
        let fractional_italic_offset = whole_italic_offset - whole_italic_offset.floor();
        let units_from_left = fractional_italic_offset * grid.spacing - (viewport.winsize.1 as f32 / viewport.factor / grid.spacing * slope).floor() * grid.spacing;

        let total_horizontal = f32::floor((viewport.winsize.0 as f32 + (viewport.winsize.0 as f32 * slope)) / viewport.factor / grid.spacing) as usize;
        for i in 0..total_horizontal {
            grid_path.move_to((units_from_left - scaled_left_offset + i as f32 * grid.spacing, -scaled_top_offset));
            grid_path.line_to(
                (units_from_left - scaled_left_offset + i as f32 * grid.spacing + viewport.winsize.1 as f32 / viewport.factor * slope,
                -scaled_top_offset + viewport.winsize.1 as f32 / viewport.factor)
            );
        }
        */
    }

    canvas.draw_path(&grid_path, &paint);
}