use super::{Handle, HandleStyle, UIPointType};
use skulpin::skia_safe::{
    Canvas, ContourMeasureIter, Font, FontMgr, FontStyle, Matrix, Paint, PaintStyle, Path, Point,
    Rect, TextBlob, Typeface, Vector,
};
pub mod calc;
use self::calc::*;

use super::constants::*;
use crate::glifparser;
use crate::state::PointLabels;
use crate::STATE;

use glifparser::Point as GlifPoint;

use std::iter::Peekable;

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
            UIPointType::Point(HandleStyle::Handlebars((Handle::At(_, _), Handle::Colocated)))
            | UIPointType::Point(HandleStyle::Handlebars((Handle::Colocated, Handle::At(_, _)))) => {
                (POINT_ONE_FILL, POINT_ONE_STROKE)
            }
            UIPointType::Point(HandleStyle::Handlebars((Handle::Colocated, Handle::Colocated)))
            | UIPointType::Direction => (POINT_SQUARE_FILL, POINT_SQUARE_STROKE),
            _ => (POINT_TWO_FILL, POINT_TWO_STROKE),
        }
    };
    (fill, stroke)
}

pub fn draw_directions(path: Path, canvas: &mut Canvas) {
    let mut piter = ContourMeasureIter::from_path(&path, false, None);
    for cm in piter {
        // Get vector and tangent -4 Skia units along the contur
        let (vec, tan) = cm.pos_tan(-4.).unwrap();
        draw_triangle_point(vec, tan, false, canvas);
    }
}

// For direction markers, not a "real" point So, we make three paths. `path` we return; `path2` is
// 90 deg offset from `path1`, both of which are lines created by elongating Skia vectors. `path2`
// is rotated at its center, such that they form an X. We elongate `path1` a bit so the final
// triangle is not isoceles. We then move to the "point" (path2[1]), make a line to the second
// point (on the base), finish that segment, and close the path.
fn draw_triangle_point(at: Point, along: Vector, selected: bool, canvas: &mut Canvas) {
    let (fill, stroke) = get_fill_and_stroke(UIPointType::Direction, selected);
    let factor = STATE.with(|v| v.borrow().factor);
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
) {
    let (fill, stroke) = get_fill_and_stroke(kind, selected);
    let factor = STATE.with(|v| v.borrow().factor);
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
) {
    let (fill, stroke) = get_fill_and_stroke(kind, selected);
    let factor = STATE.with(|v| v.borrow().factor);
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

fn draw_point_number(at: (f32, f32), number: isize, canvas: &mut Canvas) {
    let converted = number.to_string();
    draw_string_at_point(at, &converted, canvas);
}

fn draw_point_location(at: (f32, f32), original: (f32, f32), canvas: &mut Canvas) {
    let converted = format!("{}, {}", original.0, original.1);
    draw_string_at_point(at, &converted, canvas);
}

fn draw_string_at_point(mut at: (f32, f32), s: &str, canvas: &mut Canvas) {
    let factor = STATE.with(|v| v.borrow().factor);
    let mut paint = Paint::default();
    paint.set_color(0xff_ff0000);
    paint.set_anti_alias(true);

    let font = Font::from_typeface_with_params(
        Typeface::from_name("", FontStyle::bold()).expect("Failed to load bold font"),
        14.0 * 1. / factor,
        1.0,
        0.0,
    );

    let blob = TextBlob::from_str(s, &font).expect(&format!("Failed to shape {}", s));

    let (scalar, rect) = font.measure_str(s, Some(&paint));

    let mut paint2 = Paint::default();
    paint2.set_color(0xaa_ffffff);
    paint2.set_anti_alias(true);
    let mut path = Path::new();
    let padding = 5.;
    at = (at.0, at.1 - (padding + 20. * (1. / factor)));
    let at_rect = Rect::from_point_and_size(at, (rect.width() + 5., rect.height() + 5.));
    path.add_rect(at_rect, None);
    path.close();
    canvas.draw_path(&path, &paint2);

    at = (
        at.0 + (padding / 2.),
        at.1 + ((padding / 2.) + 10. * (1. / factor)),
    );
    canvas.draw_text_blob(&blob, at, &paint);
}

fn draw_point(
    at: (f32, f32),
    original: (f32, f32),
    number: Option<isize>,
    kind: UIPointType,
    selected: bool,
    canvas: &mut Canvas,
) {
    let mut paint = Paint::default();
    let factor = STATE.with(|v| v.borrow().factor);
    paint.set_anti_alias(true);
    paint.set_style(PaintStyle::StrokeAndFill);
    let thiccness = if kind == UIPointType::Handle {HANDLE_STROKE_THICKNESS} else {POINT_STROKE_THICKNESS};
    paint.set_stroke_width(thiccness * (1. / factor));
    let radius = if kind == UIPointType::Handle {HANDLE_RADIUS} else {POINT_RADIUS} * (1. / factor);

    match kind {
        UIPointType::Handle
        | UIPointType::Point(HandleStyle::Handlebars((Handle::At(_, _), Handle::At(_, _)))) => {
            draw_round_point(at, kind, selected, canvas, &mut paint);
        }
        UIPointType::Point(_) => {
            draw_square_point(at, kind, selected, canvas, &mut paint);
        }
        _ => {}
    }

    match number {
        None => {}
        Some(i) => match STATE.with(|v| v.borrow().point_labels) {
            PointLabels::None => {}
            PointLabels::Numbered => draw_point_number(at, i, canvas),
            PointLabels::Locations => draw_point_location(at, original, canvas),
        },
    }

    if let UIPointType::Point(HandleStyle::Handlebars((a, b))) = kind {
        draw_handle(a, selected, canvas);
        draw_handle(b, selected, canvas);
    }
}

fn draw_handle(h: Handle, selected: bool, canvas: &mut Canvas) {
    match h {
        Handle::Colocated => {}
        Handle::At(x, y) => {
            draw_point(
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
    prevpoint: Option<&glifparser::Point<T>>, // None in cubic mode when selecting as no access to prevpoints
    point: &glifparser::Point<T>,
    selected: bool,
    canvas: &mut Canvas,
) {
    let mut path = Path::new();
    let mut paint = Paint::default();
    let factor = STATE.with(|v| v.borrow().factor);

    paint.set_anti_alias(true);
    paint.set_color(if selected {
        SELECTED_FILL
    } else {
        HANDLEBAR_STROKE
    });
    paint.set_stroke_width(HANDLEBAR_THICKNESS * (1. / factor));
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
    if point.ptype == glifparser::PointType::QCurve {
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
    point: &glifparser::Point<T>,
    number: Option<isize>,
    selected: bool,
    canvas: &mut Canvas,
) {
    draw_point(
        (calc_x(point.x), calc_y(point.y)),
        (point.x, point.y),
        number,
        UIPointType::Point(HandleStyle::Handlebars((point.a, point.b))),
        selected,
        canvas,
    );
}
