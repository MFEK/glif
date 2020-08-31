use super::{Handle, HandleStyle, UIPointType};
use reclutch::skia::{
    Canvas, ContourMeasureIter, Matrix, Paint, PaintStyle, Path, Point, Rect, Vector,
};
pub mod calc;
use self::calc::*;

use super::constants::*;
use crate::glifparser;
use crate::state::state;

use glifparser::Point as GlifPoint;

impl From<&GlifPoint> for Point {
    fn from(p: &GlifPoint) -> Self {
        Point::from((calc_x(p.x), calc_y(p.y)))
    }
}

pub fn draw_directions(path: Path, canvas: &mut Canvas) {
    let mut piter = ContourMeasureIter::from_path(&path, false, None);
    for (cm) in piter {
        let (vec, mut tan) = cm.pos_tan(4.).unwrap();
        println!("{:?} {:?}", vec, tan);
        draw_triangle_point(vec, tan, false, canvas);
    }
}

// For direction markers, not a "real" point So, we make three paths. `path` we return; `path2` is
// 90 deg offset from `path1`, both of which are lines created by elongating Skia vectors. `path2`
// is rotated at its center, such that they form an X. We elongate `path1` a bit so the final
// triangle is not isoceles. We then move to the "point" (path2[1]), make a line to the second
// point (on the base), finish that segment, and close the path.
fn draw_triangle_point(at: Point, along: Vector, selected: bool, canvas: &mut Canvas) {
    let factor = state.with(|v| v.borrow().factor);
    let mut paint = Paint::default();
    paint.set_stroke_width(DIRECTION_STROKE_THICKNESS * (1. / factor));
    paint.set_anti_alias(true);

    let mut path = Path::new();
    let mut path1 = Path::new();

    let mut vec = along.clone();
    vec.set_length(6.0 * (1. / factor));

    let mut matrix = Matrix::new_identity();
    matrix.set_rotate(90., at + vec);

    vec.set_length(15.0 * (1. / factor));

    path1.move_to(at + vec);
    path1.line_to(at);
    let mut path2 = Path::new();
    //vec.set_length(10.);
    vec.set_length(12.0 * (1. / factor));
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
    paint.set_color(POINT_SQUARE_FILL);
    canvas.draw_path(&path, &paint);
    paint.set_style(PaintStyle::Stroke);
    paint.set_color(POINT_SQUARE_STROKE);
    canvas.draw_path(&path, &paint);
}

fn draw_round_point(
    at: (f32, f32),
    kind: UIPointType,
    selected: bool,
    canvas: &mut Canvas,
    paint: &mut Paint,
) {
    let stroke = if selected {
        SELECTED_STROKE
    } else {
        if kind != UIPointType::Point {
            HANDLE_STROKE
        } else {
            POINT_TWO_STROKE
        }
    };
    let fill = if selected {
        SELECTED_FILL
    } else {
        if kind != UIPointType::Point {
            HANDLE_FILL
        } else {
            POINT_TWO_FILL
        }
    };
    let factor = state.with(|v| v.borrow().factor);
    let radius = POINT_RADIUS
        * (1. / factor)
        * if selected && kind == UIPointType::Point {
            2.
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
    stroke: u32,
    fill: u32,
    canvas: &mut Canvas,
    paint: &mut Paint,
) {
    let factor = state.with(|v| v.borrow().factor);
    let radius = (POINT_RADIUS * (1. / factor)) * 2. * if selected { 2. } else { 1. };

    let mut path = Path::new();
    match kind {
        UIPointType::Point => {
            paint.set_color(fill);
        }
        _ => unreachable!("Handle not round"),
    }
    path.add_rect(
        Rect::from_point_and_size((at.0 - radius / 2., at.1 - radius / 2.), (radius, radius)),
        None,
    );
    path.close();
    canvas.draw_path(&path, &paint);
    match kind {
        UIPointType::Point => {
            paint.set_color(stroke);
        }
        _ => unreachable!("Handle not round"),
    }
    paint.set_style(PaintStyle::Stroke);
    canvas.draw_path(&path, &paint);
}

fn draw_point(
    at: (f32, f32),
    kind: UIPointType,
    handles: (Handle, Handle),
    selected: bool,
    canvas: &mut Canvas,
) {
    let mut paint = Paint::default();
    let factor = state.with(|v| v.borrow().factor);
    paint.set_anti_alias(true);
    paint.set_style(PaintStyle::StrokeAndFill);
    paint.set_stroke_width(POINT_STROKE_THICKNESS * (1. / factor));
    let radius = POINT_RADIUS * (1. / factor);

    match handles {
        (Handle::At(_, _), Handle::At(_, _)) => {
            draw_round_point(at, kind, selected, canvas, &mut paint);
        }
        (Handle::Colocated, Handle::At(_, _)) | (Handle::At(_, _), Handle::Colocated) => {
            draw_square_point(
                at,
                kind,
                selected,
                if selected {
                    SELECTED_STROKE
                } else {
                    POINT_ONE_STROKE
                },
                if selected {
                    SELECTED_FILL
                } else {
                    POINT_ONE_FILL
                },
                canvas,
                &mut paint,
            );
        }
        _ => {
            draw_square_point(
                at,
                kind,
                selected,
                if selected {
                    SELECTED_STROKE
                } else {
                    POINT_SQUARE_STROKE
                },
                if selected {
                    SELECTED_FILL
                } else {
                    POINT_SQUARE_FILL
                },
                canvas,
                &mut paint,
            );
        }
    }
}

fn draw_handle(h: Handle, selected: bool, canvas: &mut Canvas) {
    match h {
        Handle::Colocated => {}
        Handle::At(x, y) => {
            draw_point(
                (calc_x(x), calc_y(y)),
                UIPointType::Handle(HandleStyle::Floating),
                (Handle::At(0., 0.), Handle::At(0., 0.)),
                selected,
                canvas,
            );
        }
    }
}

fn draw_handlebars(a: Handle, b: Handle, at: (f32, f32), selected: bool, canvas: &mut Canvas) {
    let mut path = Path::new();
    let mut paint = Paint::default();
    let factor = state.with(|v| v.borrow().factor);

    paint.set_anti_alias(true);
    paint.set_color(if selected {
        SELECTED_FILL
    } else {
        HANDLEBAR_STROKE
    });
    paint.set_stroke_width(HANDLEBAR_THICKNESS * (1. / factor));
    paint.set_style(PaintStyle::Stroke);

    match a {
        Handle::At(x, y) => {
            path.move_to((calc_x(x), calc_y(y)));
            path.line_to((at.0, at.1));
        }
        _ => {
            path.move_to((at.0, at.1));
        }
    }
    match b {
        Handle::At(x, y) => {
            path.line_to((calc_x(x), calc_y(y)));
        }
        _ => {}
    }
    canvas.draw_path(&path, &paint);
}

pub fn draw_complete_point(
    point: &glifparser::Point,
    kind: UIPointType,
    selected: bool,
    canvas: &mut Canvas,
) {
    draw_handlebars(
        point.a,
        point.b,
        (calc_x(point.x), calc_y(point.y)),
        selected,
        canvas,
    );
    draw_point(
        (calc_x(point.x), calc_y(point.y)),
        UIPointType::Point,
        (point.a, point.b),
        selected,
        canvas,
    );
    draw_handle(point.a, selected, canvas);
    draw_handle(point.b, selected, canvas);
}
