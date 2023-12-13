use MFEKmath::mfek::ResolveCubic;
use MFEKmath::quadbezier::QuadBezier;
use MFEKmath::subdivide::Subdivide;
use flo_curves::bezier::curve_intersects_line;
use flo_curves::{
    bezier::Curve as FloCurve,
    geo::Coord2,
};
use flo_curves::{BezierCurveFactory, Line, Coordinate2D};
use glifparser::MFEKPointData;
use glifparser::glif::contour_operations::ContourOperation;
use glifparser::glif::inner::MFEKContourInner;
use glifparser::glif::{MFEKOutline, MFEKContour};
use glifparser::glif::contour::MFEKContourCommon;
use glifrenderer::constants::{self, OUTLINE_STROKE_THICKNESS};
use kurbo::{PathSeg, QuadBez, ParamCurve};
use skia_safe::{Canvas, Paint, Path, Point};
use MFEKmath::{Piecewise, Bezier, Evaluate};

use crate::editor::Editor;
use crate::user_interface::Interface;

use super::prelude::*;

#[derive(Clone, Debug)]
pub struct Cut {
    start_point: Option<(f32, f32)>,
}

pub struct Intersection {
    pub ci: usize,
    pub bi: usize,
    pub t: f64,
    pub line_t: f64,
    pub coords: (f64, f64),
}

impl Tool for Cut {
    #[rustfmt::skip]
    fn event(&mut self, v: &mut Editor, _i: &mut Interface, event: EditorEvent) {
        if let EditorEvent::MouseEvent { mouse_info, event_type } = event {
            match event_type {
                MouseEventType::Pressed => self.mouse_pressed(v, mouse_info),
                MouseEventType::Released => self.mouse_released(v, mouse_info),
                _ => (),
            }
        }
    }

    fn draw(&mut self, v: &Editor, i: &Interface, canvas: &mut Canvas) {
        Self::draw_line(i, v, canvas, &self.start_point, &Some(i.mouse_info.position));
    }
}

impl Cut {
    pub fn new() -> Self {
        Self { start_point: None }
    }

    fn mouse_pressed(&mut self, _v: &Editor, mouse_info: MouseInfo) {
        self.start_point = Some(mouse_info.position);
    }

    fn mouse_released(&mut self, v: &mut Editor, mouse_info: MouseInfo) {        
        let intersections = Self::find_intersections(&self.start_point, &Some(mouse_info.position.into()), v, false);
        Self::split_at_intersections(v, &intersections);
        self.start_point = None;
    }

    pub fn find_intersections(start_point: &Option<(f32, f32)>, end_point: &Option<(f32, f32)>, v: &Editor, non_cubic: bool) -> Vec<Intersection> {
        let mut intersections = vec![];
        if start_point.is_none() || end_point.is_none() { return intersections }

        let start_point = start_point.unwrap();
        let end_point = end_point.unwrap();

        fn intersect_cubic(pw: Piecewise<Bezier>, ci: usize, intersections: &mut Vec<Intersection>, sp: (f32, f32), ep: (f32, f32)) {
            for (bi, bez) in pw.segs.iter().enumerate() {
                let flo_bez = FloCurve::from_points(
                    Coord2(bez.w1.x, bez.w1.y),
                    (
                        Coord2(bez.w2.x, bez.w2.y),
                        Coord2(bez.w3.x, bez.w3.y),
                    ),
                    Coord2(bez.w4.x, bez.w4.y),
                );

                let flo_line = <(Coord2, Coord2) as Line>::from_points(sp.into(), ep.into());

                for (curve_t, line_t, coordinate) in curve_intersects_line(&flo_bez, &flo_line) {
                    let intersection = Intersection {
                        ci: ci,
                        t: curve_t,
                        line_t: line_t,
                        coords: (coordinate.x(), coordinate.y()),
                        bi: bi,
                    };

                    intersections.push(intersection);
                }
            }
        }

        // first we've gotta look at the active layer and find intersections between the line we drew
        // and the contours of the active layer
        for (ci, c) in v.get_active_layer_ref().outline.iter().enumerate() {
            match c.get_type() {
                glifparser::glif::inner::MFEKContourInnerType::Cubic => {
                    let pw: Piecewise<Bezier> = Piecewise::from(c.cubic().unwrap());
                    intersect_cubic(pw, ci, &mut intersections, start_point, end_point);
                },
                glifparser::glif::inner::MFEKContourInnerType::Quad => {
                    let start_point = (start_point.0 as f64, start_point.1 as f64);
                    let end_point = (end_point.0 as f64, end_point.1 as f64);
                    let pw: Piecewise<QuadBezier> = Piecewise::from(c.quad().unwrap());

                    for (bi, bez) in pw.segs.iter().enumerate() {
                        let quadbez = QuadBez::new(bez.w1, bez.w2, bez.w3);
                        let quad = PathSeg::Quad(quadbez);
                        let line = kurbo::Line::new(start_point, end_point);
                        let line_intersections = PathSeg::intersect_line(&quad, line);

                        for li in line_intersections {
                            let line_t = li.line_t;
                            let curve_t = li.segment_t;

                            let coordinate = quadbez.eval(curve_t);

                            let intersection = Intersection {
                                ci: ci,
                                t: curve_t,
                                line_t: line_t,
                                coords: (coordinate.x, coordinate.y),
                                bi: bi,
                            };
    
                            intersections.push(intersection);
                        }
                    }
                },
                _ => {
                    let pw: Piecewise<Bezier> = Piecewise::from(c.to_cubic().cubic().unwrap());
                    intersect_cubic(pw, ci, &mut intersections, start_point, end_point);
                },
            }
        };

        intersections.sort_by(|a, b| a.line_t.partial_cmp(&b.line_t).unwrap_or(std::cmp::Ordering::Equal));

        intersections

    }

    pub fn split_at_intersections(v: &mut Editor, intersections: &[Intersection]) {
        if intersections.len() == 0 { return }
        let mut new_outline = MFEKOutline::new();
    
        for (ci, contour) in v.get_active_layer_ref().outline.iter().enumerate() {
            let mut added_cuts = Vec::new();
            let previous_operation = contour.operation.clone();

            match contour.get_type() {
                glifparser::glif::inner::MFEKContourInnerType::Cubic => {
                    let mut new_beziers = Vec::new();
                    let pw: Piecewise<Bezier> = Piecewise::from(contour.cubic().unwrap());
                
                    for (bi, bez) in pw.segs.iter().enumerate() {
                        // Find intersections relevant for the current Bezier
                        let split_times_for_bezier: Vec<_> = intersections.iter()
                            .filter(|&intersection| {
                                intersection.ci == ci && intersection.bi == bi
                            })
                            .map(|intersection| intersection.t)
                            .collect();
        
                        if !split_times_for_bezier.is_empty() {
                            let mut sorted_times = split_times_for_bezier.clone();
                            sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
                            let split_beziers = bez.split_at_multiple_t(sorted_times);
                            
    
                            for _ in 0 .. split_times_for_bezier.len() {
                                added_cuts.push(bi);
                            }
    
                            // Add the split beziers to the new beziers list
                            new_beziers.extend(split_beziers);
                        } else {
                            // If there were no intersections for this bezier, just add it as is
                            new_beziers.push(bez.clone());
                        }
                    }


                    // Construct a new contour from the new beziers
                    let mut new_contour: MFEKContour<MFEKPointData> = Piecewise::new(new_beziers, None).to_contour().into();
                    new_contour.set_operation(previous_operation);

                    for cut in added_cuts {
                        new_contour.operation_mut().insert_op(cut)
                    }

                    new_outline.push(new_contour);
                },
                glifparser::glif::inner::MFEKContourInnerType::Quad => {
                    let mut new_beziers = Vec::new();
                    let pw: Piecewise<QuadBezier> = Piecewise::from(contour.quad().unwrap());
                
                    for (bi, bez) in pw.segs.iter().enumerate() {
                        // Find intersections relevant for the current Bezier
                        let split_times_for_bezier: Vec<_> = intersections.iter()
                            .filter(|&intersection| {
                                intersection.ci == ci && intersection.bi == bi
                            })
                            .map(|intersection| intersection.t)
                            .collect();
        
                        if !split_times_for_bezier.is_empty() {
                            let mut sorted_times = split_times_for_bezier.clone();
                            sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
                            let split_beziers = bez.split_at_multiple_t(sorted_times);
                            
    
                            for _ in 0 .. split_times_for_bezier.len() {
                                added_cuts.push(bi);
                            }
    
                            // Add the split beziers to the new beziers list
                            new_beziers.extend(split_beziers);
                        } else {
                            // If there were no intersections for this bezier, just add it as is
                            new_beziers.push(bez.clone());
                        }
                    }


                    // Construct a new contour from the new beziers
                    let new_contour = Piecewise::new(new_beziers, None).to_contour();
                    let mut mfek_contour = MFEKContour::new(MFEKContourInner::Quad(new_contour), previous_operation);
                    
                    match contour.is_closed() {
                        true => mfek_contour.set_closed(),
                        false => mfek_contour.set_open(),
                    }

                    for cut in added_cuts {
                        mfek_contour.operation_mut().insert_op(cut)
                    }

                    new_outline.push(mfek_contour);
                },
                glifparser::glif::inner::MFEKContourInnerType::Hyper => {},
            }
        }

        v.begin_modification("Cut", false);
        v.get_active_layer_mut().outline = new_outline;
        v.end_modification();
    }
    

    pub fn draw_line(i: &Interface, v: &Editor, canvas: &mut Canvas, start_point: &Option<(f32, f32)>, end_point: &Option<(f32, f32)>) {
        let mut path = Path::new();
        let mut paint = Paint::default();
        let factor = i.viewport.factor;
    
        if let (Some(measure_from), Some(measure_to)) = (start_point, end_point) {
            let skpath_start = Point::new(measure_from.0 as f32, measure_from.1 as f32);
            let skpath_end = Point::new(
                measure_to.0 as f32,
                measure_to.1 as f32,
            );
    
            path.move_to(skpath_start);
            path.line_to(skpath_end);
    
            paint.set_color(constants::MEASURE_STROKE);
            paint.set_anti_alias(true);
            paint.set_style(skia_safe::PaintStyle::Stroke);
            paint.set_stroke_width(OUTLINE_STROKE_THICKNESS * (1. / factor));
            canvas.draw_path(&path, &paint);
    
            // Draw X's at the intersections
            let intersections = Self::find_intersections(&start_point, &end_point, v, true); // Assuming you have access to the editor here
            for intersection in intersections {
                let intersection_point = Point::new(intersection.coords.0 as f32, intersection.coords.1 as f32);
                
                let x_size = 10.0 / factor; // Adjust this value for the desired size of the X
    
                // Drawing the two lines of the X
                path.reset();
                let start1 = Point::new(intersection_point.x - x_size, intersection_point.y - x_size);
                let end1 = Point::new(intersection_point.x + x_size, intersection_point.y + x_size);
                path.move_to(start1);
                path.line_to(end1);
    
                let start2 = Point::new(intersection_point.x + x_size, intersection_point.y - x_size);
                let end2 = Point::new(intersection_point.x - x_size, intersection_point.y + x_size);
                path.move_to(start2);
                path.line_to(end2);
                
                canvas.draw_path(&path, &paint);
            }
        }
    }
}