use super::{Handle, UIPointType};
use skulpin::skia_safe::{
    Canvas, ContourMeasureIter, Matrix, Paint, PaintStyle, Path, Point, Rect, Vector,
};
pub mod calc;
use self::calc::*;
pub mod names;

use super::constants::*;
use crate::{glifparser, editor::Editor};
use crate::editor::{HandleStyle, PointLabels};

use glifparser::{Point as GlifPoint, PointType};

type Color = u32;

trait SkiaFromGlyph<T> {
    fn from_glif(p: &GlifPoint<T>) -> Point;
}

impl<T> SkiaFromGlyph<T> for Point {
    fn from_glif(p: &GlifPoint<T>) -> Self {
        Point::from((calc_x(p.x), calc_y(p.y)))
    }
}

fn get_fill_and_stroke(kind: UIPointType, selected: bool) -> (Color, Color) {
    let (fill, stroke) = if selected {
        (SELECTED_FILL, SELECTED_STROKE)
    } else {
        match kind {
            UIPointType::Handle => (HANDLE_FILL, HANDLE_STROKE),
            UIPointType::Point((Handle::At(_, _), Handle::Colocated))
            | UIPointType::Point((Handle::Colocated, Handle::At(_, _))) => {
                (POINT_ONE_FILL, POINT_ONE_STROKE)
            }
            UIPointType::Point((Handle::Colocated, Handle::Colocated)) | UIPointType::Direction => {
                (POINT_SQUARE_FILL, POINT_SQUARE_STROKE)
            }
            _ => (POINT_TWO_FILL, POINT_TWO_STROKE),
        }
    };
    (fill, stroke)
}

pub fn draw_directions(v: &Editor, path: Path, canvas: &mut Canvas) {
    let piter = ContourMeasureIter::from_path(&path, false, None);
    for cm in piter {
        // Get vector and tangent -4 Skia units along the contur
        let (vec, tan) = cm.pos_tan(-4.).unwrap();
        draw_triangle_point(v, vec, tan, false, canvas);
    }
}

// For direction markers, not a "real" point So, we make three paths. `path` we return; `path2` is
// 90 deg offset from `path1`, both of which are lines created by elongating Skia vectors. `path2`
// is rotated at its center, such that they form an X. We elongate `path1` a bit so the final
// triangle is not isoceles. We then move to the "point" (path2[1]), make a line to the second
// point (on the base), finish that segment, and close the path.
fn draw_triangle_point(v: &Editor, at: Point, along: Vector, selected: bool, canvas: &mut Canvas) {
    let (fill, stroke) = get_fill_and_stroke(UIPointType::Direction, selected);
    let factor = v.viewport.factor;
    let mut paint = Paint::default();
    paint.set_stroke_width(DIRECTION_STROKE_THICKNESS * (1. / factor));
    paint.set_anti_alias(true);

    let mut path = Path::new();
    let mut path1 = Path::new();

    let mut vec = along.clone();
    vec.set_length(TRIANGLE_POINT_AREA * (1. / factor));

    let mut matrix = Matrix::new_identity();
    matrix.set_rotate(90., at + vec);

    vec.set_length(TRIANGLE_POINT_AREA * 2.5 * (1. / factor));

    path1.move_to(at + vec);
    path1.line_to(at);
    let mut path2 = Path::new();
    //vec.set_length(10.);
    vec.set_length(TRIANGLE_POINT_AREA * 2. * (1. / factor));
    path2.move_to(at + vec);
    path2.line_to(at);
    path2.transform(&matrix);

    let points1_count = path1.count_points();
    let mut points1 = vec![Point::default(); points1_count];
    path1.get_points(&mut points1);

    let points2_count = path2.count_points();
    let mut points2 = vec![Point::default(); points2_count];
    path2.get_points(&mut points2);

    path.move_to(points2[1]);
    path.line_to(points2[0]);
    path.line_to(points1[0]);
    path.close();

    paint.set_style(PaintStyle::StrokeAndFill);
    paint.set_color(fill);
    canvas.draw_path(&path, &paint);
    paint.set_style(PaintStyle::Stroke);
    paint.set_color(stroke);
    canvas.draw_path(&path, &paint);
}

fn draw_round_point(
    at: (f32, f32),
    kind: UIPointType,
    selected: bool,
    canvas: &mut Canvas,
    paint: &mut Paint,
    factor: f32,
) {
    let (fill, stroke) = get_fill_and_stroke(kind, selected);
    let factor = factor;
    let radius = POINT_RADIUS
        * (1. / factor)
        * if kind != UIPointType::Handle && selected {
            1.75
        } else {
            1.
        };
    paint.set_color(fill);
    canvas.draw_circle((at.0, at.1), radius, &paint);
    paint.set_style(PaintStyle::Stroke);
    paint.set_color(stroke);
    canvas.draw_circle((at.0, at.1), radius, &paint);
}

fn draw_square_point(
    at: (f32, f32),
    kind: UIPointType,
    selected: bool,
    canvas: &mut Canvas,
    paint: &mut Paint,
    factor: f32
) {
    let (fill, stroke) = get_fill_and_stroke(kind, selected);
    let radius = (POINT_RADIUS * (1. / factor)) * 2. * if selected { 1.75 } else { 1. };

    let mut path = Path::new();
    paint.set_color(fill);
    path.add_rect(
        Rect::from_point_and_size((at.0 - radius / 2., at.1 - radius / 2.), (radius, radius)),
        None,
    );
    path.close();
    canvas.draw_path(&path, &paint);
    paint.set_color(stroke);
    paint.set_style(PaintStyle::Stroke);
    canvas.draw_path(&path, &paint);
}

pub fn draw_point(
    v: &Editor,
    at: (f32, f32),
    original: (f32, f32),
    number: Option<isize>,
    kind: UIPointType,
    selected: bool,
    canvas: &mut Canvas,
) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(PaintStyle::StrokeAndFill);
    let thiccness = if kind == UIPointType::Handle {
        HANDLE_STROKE_THICKNESS
    } else {
        POINT_STROKE_THICKNESS
    };
    paint.set_stroke_width(thiccness * (1. / v.viewport.factor));
    let _radius = if kind == UIPointType::Handle {
        HANDLE_RADIUS
    } else {
        POINT_RADIUS
    } * (1. / v.viewport.factor);

    match kind {
        UIPointType::Handle | UIPointType::Point((Handle::At(_, _), Handle::At(_, _))) => {
            draw_round_point(at, kind, selected, canvas, &mut paint, v.viewport.factor);
        }
        UIPointType::Point(_) => {
            draw_square_point(at, kind, selected, canvas, &mut paint, v.viewport.factor);
        }
        _ => {}
    }

    match number {
        None => {}
        Some(i) => match v.viewport.point_labels {
            PointLabels::None => {}
            PointLabels::Numbered => names::draw_point_number(v, at, i, canvas),
            PointLabels::Locations => names::draw_point_location(v, at, original, canvas),
        },
    }

    if let UIPointType::Point((a, b)) = kind {
        if v.viewport.handle_style != HandleStyle::None {
            draw_handle(v, a, selected, canvas);
            draw_handle(v, b, selected, canvas);
        }
    }
}

fn draw_handle(v: &Editor, h: Handle, selected: bool, canvas: &mut Canvas) {

    match h {
        Handle::Colocated => {}
        Handle::At(x, y) => {
            draw_point(
                v,
                (calc_x(x), calc_y(y)),
                (x, y),
                None,
                UIPointType::Handle,
                selected,
                canvas,
            );
        }
    }
}

pub fn draw_handlebars<T>(
    v: &Editor,
    prevpoint: Option<&glifparser::Point<T>>, // None in cubic mode when selecting as no access to prevpoints
    point: &glifparser::Point<T>,
    selected: bool,
    canvas: &mut Canvas,
) {
    let mut path = Path::new();
    let mut paint = Paint::default();

    paint.set_anti_alias(true);
    paint.set_color(if selected {
        SELECTED_FILL
    } else {
        HANDLEBAR_STROKE
    });
    paint.set_stroke_width(HANDLEBAR_THICKNESS * (1. / v.viewport.factor));
    paint.set_style(PaintStyle::Stroke);

    match point.a {
        Handle::At(x, y) => {
            path.move_to((calc_x(x), calc_y(y)));
            path.line_to((calc_x(point.x), calc_y(point.y)));
        }
        _ => {
            path.move_to((calc_x(point.x), calc_y(point.y)));
        }
    }
    match point.b {
        Handle::At(x, y) => {
            path.line_to((calc_x(x), calc_y(y)));
        }
        _ => {}
    }
    if point.ptype == PointType::QCurve || point.ptype == PointType::QClose {
        if let Some(pp) = prevpoint {
            match pp.a {
                Handle::At(x, y) => {
                    path.line_to((calc_x(x), calc_y(y)));
                }
                _ => {}
            }
        }
    }
    canvas.draw_path(&path, &paint);
}

pub fn draw_complete_point<T>(
    v: &Editor,
    point: &glifparser::Point<T>,
    number: Option<isize>,
    selected: bool,
    canvas: &mut Canvas,
) {
    if point.ptype == PointType::QClose {
        return;
    }

    draw_point(
        v,
        (calc_x(point.x), calc_y(point.y)),
        (point.x, point.y),
        number,
        UIPointType::Point((point.a, point.b)),
        selected,
        canvas,
    );
}

pub fn draw_all(v: &Editor, canvas: &mut Canvas) {
    let mut i: isize = -1;
    let handle_style = v.viewport.handle_style;

    // FIXME: this is bad but I can't access v from inside with glif
    let selected = v.selected.clone();
    let active_layer = v.get_active_layer();
    let (vcidx, vpidx) = (v.contour_idx, v.point_idx);
    v.with_glyph(|glif| {
        for (lidx, layer) in glif.layers.iter().enumerate() {
            if lidx != active_layer { continue };
            if handle_style == HandleStyle::Handlebars {
                for (cidx, contour) in layer.outline.as_ref().unwrap() .iter().enumerate(){
                    let mut prevpoint = contour.first().unwrap();
                    for (pidx, point) in contour.iter().enumerate() {
                        let selected = if  
                            (lidx == active_layer && selected.contains(&(cidx, pidx))) ||
                            (lidx == active_layer && vcidx == Some(cidx) && vpidx == Some(pidx))
                        { true } else { false };
                        draw_handlebars(v, Some(prevpoint), &point, selected, canvas);
                        prevpoint = &point;
                    }
                }
            }

            for (cidx, contour) in layer.outline.as_ref().unwrap() .iter().enumerate(){
                for (pidx, point) in contour.iter().enumerate() {
                    if point.b != Handle::Colocated {
                        i += 1;
                    }
                    let selected = if  
                        (lidx == active_layer && selected.contains(&(cidx, pidx))) ||
                        (lidx == active_layer && vcidx == Some(cidx) && vpidx == Some(pidx))
                    { true } else { false };
                
                    draw_complete_point(v, &point, Some(i), selected, canvas);
                    if point.a != Handle::Colocated {
                        i += 1;
                    }
                    i += 1;
                }
            }
        }
    });
}