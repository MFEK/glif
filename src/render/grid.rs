use glifrenderer::calc_x;
use glifrenderer::calc_y;
use glifrenderer::constants::GUIDELINE_THICKNESS;
use glifrenderer::viewport::Viewport;
use skulpin::skia_safe::Canvas;
use skulpin::skia_safe::Color4f;
use skulpin::skia_safe::Paint;
use skulpin::skia_safe::PaintStyle;
use skulpin::skia_safe::Path;

use crate::user_interface::grid::Grid;

pub fn draw_grid(canvas: &mut Canvas, grid: &Grid, viewport: &Viewport) {
    let mut grid_path = Path::new();
    let slope = grid.slope.filter(|gs|*gs != 0. && !gs.is_subnormal()).unwrap_or(0.);

    let dmatrix = viewport.as_device_matrix();
    let offset  = dmatrix.map_origin();
    let factor = dmatrix.scale_x().abs();
    let offset  = (offset.x * factor, offset.y * factor);

    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_stroke_width(GUIDELINE_THICKNESS / factor);
    paint.set_style(PaintStyle::Stroke);
    paint.set_color4f(Color4f::new(0., 0., 0., 1.0), None);

    let mut scaled_left_offset = offset.0 / 2.;
    let mut scaled_top_offset = offset.1 / 2.;

    let whole_left_offset = (grid.offset + scaled_left_offset) / grid.spacing;
    let fractional_left_offset = if slope > 0. {
        whole_left_offset - whole_left_offset.floor()
    } else {
        whole_left_offset.ceil() - whole_left_offset
    };
    let units_from_left = fractional_left_offset * grid.spacing;

    let total_vertical =
        (f32::ceil((viewport.winsize.0 as f32 * (1. / viewport.factor))) / grid.spacing) as i32;
    for i in 0..total_vertical {
        grid_path.move_to((
            calc_x(units_from_left - scaled_left_offset + (i as f32 * grid.spacing)),
            -scaled_top_offset,
        ));
        grid_path.line_to((
            calc_x(units_from_left - scaled_left_offset + (i as f32 * grid.spacing)),
            -scaled_top_offset + (viewport.winsize.1 as f32 * (1. / viewport.factor)),
        ));
    }

    let whole_top_offset = (grid.offset - scaled_top_offset) / grid.spacing;
    let fractional_top_offset = whole_top_offset - whole_top_offset.floor();
    let units_from_top = fractional_top_offset * grid.spacing;

    let total_horizontal =
        f32::floor(viewport.winsize.1 as f32 / viewport.factor / grid.spacing) as i32;
    for i in -total_horizontal..total_horizontal {
        grid_path.move_to((
            calc_x(-scaled_left_offset),
            calc_y(units_from_top) - scaled_top_offset + i as f32 * grid.spacing,
        ));
        grid_path.line_to((
            calc_x(-scaled_left_offset + viewport.winsize.0 as f32 / viewport.factor),
            calc_y(units_from_top) - scaled_top_offset + i as f32 * grid.spacing,
        ));
    }

    // Draw slanted lines (italic / rotalic)

    if slope == 0. || slope.is_subnormal() {
        return;
    }
    /*if grid.slope.map(|gs|gs < 0.).unwrap_or(false) {
        scaled_left_offset = -scaled_left_offset;
    }*/

    let slope_max: f32 = if slope < 0. { f32::min(-1., slope) } else { f32::max(1., slope) };

    let viewx = viewport.winsize.0 as f32 / viewport.factor;

    let spacing = grid.spacing * slope_max;
    let extra = -total_horizontal + (-total_horizontal as f32 * slope.abs()) as i32;
    let offset = ((grid.offset + scaled_top_offset + scaled_left_offset * slope.abs()) / spacing)
        .fract()
        * spacing;

    for i in extra..-extra {
        grid_path.move_to((
            calc_x(scaled_left_offset),
            calc_y(spacing * i as f32 - offset + scaled_top_offset),
        ));
        grid_path.line_to((
            calc_x(viewx + scaled_left_offset),
            calc_y(slope * (viewx) + spacing * i as f32 - offset + scaled_top_offset),
        ));
    }

    canvas.draw_path(&grid_path, &paint);
}
