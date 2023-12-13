mod dialog;

use std::collections::HashMap;
use std::f32::consts::PI;

use super::prelude::*;
use crate::tool_behaviors::draw_pivot::DrawPivot;
use crate::tool_behaviors::selection_box::SelectionBox;
use crate::user_interface::Interface;

use float_cmp::ApproxEq;
use glifparser::MFEKPointData;
use glifparser::outline::{skia::FromSkiaPath as _, FromKurbo as _, Reverse};
use glifparser::{glif::MFEKContour, Outline};
use kurbo;
use kurbo::Shape as _;
use num;
use num_derive::FromPrimitive;
use skia_safe::{Matrix, Path, PathDirection, PathEffect, StrokeRec};


#[derive(Clone, Debug)]
pub struct Shapes {
    pressed_pos: Option<(f32, f32)>,
    dropped_shape: bool,
    // Because of imgui, we can't have associated types on the ShapeType enum. Thus, this
    // compromise.
    stype: ShapeType,
    sdata: ShapeData,
    corners: Option<((f32, f32), (f32, f32))>,
    draw_pivot: DrawPivot,
    locked_angle: bool,

    //ui
    edit_buf: HashMap<String, String>,
}

impl Tool for Shapes {
    fn event(&mut self, v: &mut Editor, _i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent {
                event_type,
                mouse_info,
            } => match event_type {
                MouseEventType::Moved => self.mouse_moved(v, mouse_info),
                MouseEventType::Pressed => self.mouse_pressed(v, mouse_info),
                MouseEventType::Released => self.mouse_released(v, mouse_info),
                _ => {}
            },
            EditorEvent::ScrollEvent { vertical, .. } => self.scroll(vertical),
            _ => {}
        }
    }

    fn dialog(&mut self, _v: &mut Editor, i: &mut Interface, ui: &mut Ui) -> bool {
        self.shape_settings(i, ui);
        true
    }

    fn draw(&mut self, v: &Editor, i: &Interface, canvas: &Canvas) {
        if let Some(corners) = self.corners {
            SelectionBox::draw_box_impl(i, canvas, corners);
        }
        if self.pressed_pos.is_some() {
            self.draw_pivot.pivot_point = self.pressed_pos;
            self.draw_pivot.draw(v, i, canvas);
        }
    }
}

// Do not modify w/o modifying ShapeType prev/next impl's!
#[derive(Clone, Copy, derive_more::Display, Debug, PartialEq, FromPrimitive)]
#[display(fmt = "{}")]
pub enum ShapeType {
    #[display(fmt = "circle")]
    Circle,
    #[display(fmt = "oval")]
    Oval,
    #[display(fmt = "rectangle")]
    Rectangle,
    #[display(fmt = "rounded rectangle")]
    RoundedRectangle,
    #[display(fmt = "polygon")]
    Polygon,
    #[display(fmt = "star")]
    Star,
}

// Implement scrolling through options
// Perhaps some day consider merging this w/trigger_toggle_on!(…)?
impl ShapeType {
    fn prev(&self) -> Self {
        use ShapeType::*;
        match self {
            Oval | Rectangle | RoundedRectangle | Polygon | Star => {
                num::FromPrimitive::from_u32(*self as u32 - 1).unwrap()
            }
            Circle => Star,
        }
    }

    fn next(&self) -> Self {
        use ShapeType::*;
        match self {
            Circle | Oval | Rectangle | RoundedRectangle | Polygon => {
                num::FromPrimitive::from_u32(*self as u32 + 1).unwrap()
            }
            Star => Circle,
        }
    }
}


#[derive(Copy, Clone, Debug, derive_more::Display)]
#[display(
    fmt = "{}{}{}",
    "self.display_sides()",
    "self.display_angle()",
    "self.display_radius()"
)]
pub struct ShapeData {
    polygon_angle: f32,
    polygon_sides: u16,
    polygon_radius: f32,
    rrect_radius: f32,
}

impl ShapeData {
    fn display_angle(&self) -> String {
        format!(" rotated {}°", self.polygon_angle)
    }
    fn display_sides(&self) -> String {
        format!("{}-sided", self.polygon_sides)
    }
    fn display_radius(&self) -> String {
        Some(self.polygon_radius)
            .into_iter()
            .filter(|f| f.is_normal() && !f.is_nan() && !f.approx_eq(0.0f32, (f32::EPSILON, 10)))
            .next()
            .map(|f| format!(" (rounded w/radii {})", f))
            .unwrap_or(String::new())
    }
}

impl Default for ShapeData {
    fn default() -> Self {
        Self {
            polygon_angle: 0.0,
            polygon_sides: 5,
            polygon_radius: 0.,
            rrect_radius: 50.,
        }
    }
}

impl Shapes {
    pub fn new() -> Self {
        Self {
            pressed_pos: None,
            dropped_shape: false,
            stype: ShapeType::Circle,
            sdata: ShapeData::default(),
            corners: None,
            draw_pivot: DrawPivot::default(),
            locked_angle: false,
            edit_buf: HashMap::new(),
        }
    }
}

struct ShapeDrawer {
    mouse_info: MouseInfo,
    from: (f32, f32),
    sdata: ShapeData,
    corners: Option<((f32, f32), (f32, f32))>,
}

fn shape_direction(cx: f32) -> PathDirection {
    if cx.is_sign_negative() {
        PathDirection::CCW
    } else {
        PathDirection::CW
    }
}

impl ShapeDrawer {
    fn calculate_radius(&self) -> (f32, f32, f32) {
        let (cx, cy) = (
            self.from.0 - self.mouse_info.position.0,
            self.from.1 - self.mouse_info.position.1,
        );
        (cx, cy, ((cx).powf(2.) + (cy).powf(2.)).sqrt())
    }

    fn draw_circle(&self) -> Outline<MFEKPointData> {
        let (cx, _cy, radius) = self.calculate_radius();
        let direction = shape_direction(cx);
        let kp_center = kurbo::Point::new(self.from.0 as f64, self.from.1 as f64);
        let circle = kurbo::BezPath::from_vec(
            kurbo::Circle::new(kp_center, radius as f64)
                .path_elements(1.0)
                .collect(),
        );
        let mut gcircle = Outline::from_kurbo(&circle);
        if direction == PathDirection::CCW {
            gcircle.reverse();
        }
        gcircle
    }

    fn polygon_angle(&self) -> f32 {
        let (cxm, cym, _radius) = self.calculate_radius();
        let rotangle = (cym / cxm).atan();
        let rotangle = rotangle * (180. / PI);
        let rotangle = if cxm > 0. { 180. + rotangle } else { rotangle };
        let rotangle = if rotangle == 90. || rotangle == -90. {
            -rotangle
        } else {
            rotangle
        };
        rotangle
    }

    // Odd even causes a pentagram at 5, and interesting connected shapes at odd numbers above 5.
    fn draw_polygon(&self, polygon: ShapeType) -> Outline<MFEKPointData> {
        let (sides, oddeven) = match polygon {
            ShapeType::Polygon => (self.sdata.polygon_sides, false),
            ShapeType::Star => (self.sdata.polygon_sides, true),
            _ => panic!("Called draw_polygon without a ShapeType::Polygon/Star!"),
        };
        let (cxm, _cym, radius) = self.calculate_radius();
        let rotangle = self.sdata.polygon_angle;
        let direction = shape_direction(cxm);
        let cx = self.from.0;
        let cy = self.from.1;
        let angle = 2. * PI / sides as f32;
        let mut path = Path::new();

        // We unfortunately must collect the ranges into Vec's, because type Rev<Range> !=
        // type Range , and also there's no way to do something like Rev<Range> -> Range
        let sides_vec;
        let range;
        if oddeven {
            range = {
                let mut evens = (0..sides + 1).filter(|e| e % 2 == 0).collect::<Vec<_>>();
                let odds = (0..sides + 1).filter(|e| e % 2 == 1).collect::<Vec<_>>();
                evens.extend(odds);
                evens
            };
        } else {
            range = (0..sides + 1).collect::<Vec<_>>();
        }

        let range_iter = match direction {
            PathDirection::CCW => {
                sides_vec = range.into_iter().rev().collect::<Vec<_>>();
                sides_vec.iter().enumerate()
            }
            PathDirection::CW => {
                sides_vec = range;
                sides_vec.iter().enumerate()
            }
        };

        for (i, side) in range_iter {
            if i == 0 {
                continue;
            }
            let pathbuild_func = if i == 1 { Path::move_to } else { Path::line_to };
            pathbuild_func(
                &mut path,
                (
                    cx + (radius * (angle * *side as f32).cos()),
                    cy + (radius * (angle * *side as f32).sin()),
                ),
            );
        }

        path.close();
        path.transform(&Matrix::rotate_deg_pivot(rotangle, self.from));
        if self.sdata.polygon_radius > 0.0 {
            let effect = PathEffect::corner_path(self.sdata.polygon_radius).unwrap();
            let (out_path, _) = effect
                .filter_path(&path, &StrokeRec::new_fill(), &path.compute_tight_bounds())
                .unwrap();
            Outline::from_skia_path(&out_path)
        } else {
            Outline::from_skia_path(&path)
        }
    }

    fn draw_fits_in_rect(&mut self, stype: ShapeType) -> Outline<MFEKPointData> {
        let (fx, fy, mx, my) = (
            self.from.0 as f64,
            self.from.1 as f64,
            self.mouse_info.position.0 as f64,
            self.mouse_info.position.1 as f64,
        );
        let mut rect = kurbo::Rect::new(fx, fy, mx, my);
        if self.mouse_info.modifiers.shift {
            let (dx, dy) = (mx - fx, my - fy);
            let (dx, dy) = (dx.abs(), dy.abs());
            let size = f64::max(dx, dy);
            rect = kurbo::Rect::from_center_size(rect.origin(), (size, -size));
        }
        let (cx, _, _) = self.calculate_radius();
        self.corners = Some((
            (rect.min_x() as f32, rect.min_y() as f32),
            (rect.max_x() as f32, rect.max_y() as f32),
        ));
        let path = kurbo::BezPath::from_vec(match stype {
            ShapeType::Oval => kurbo::Ellipse::from_rect(rect).path_elements(1.0).collect(),
            ShapeType::RoundedRectangle => {
                kurbo::RoundedRect::from_rect(rect, self.sdata.rrect_radius as f64)
                    .path_elements(1.0)
                    .collect()
            }
            ShapeType::Rectangle => {
                self.corners = None; // pointless
                rect.path_elements(1.0).collect()
            }
            _ => unimplemented!("ShapeType doesn't fit in rect, or unimplemented"),
        });
        let mut ret = Outline::from_kurbo(&path);
        if cx.is_sign_negative() {
            ret.reverse();
        }
        ret
    }
}

impl Shapes {
    fn describe_history_entry(&self) -> String {
        let shape_type = self.stype;
        let shape_data = if self.stype == ShapeType::Polygon || self.stype == ShapeType::Star {
            self.sdata.to_string()
        } else {
            String::new()
        };
        format!("Drew {} ({}).", shape_type, shape_data)
    }

    fn mouse_pressed(&mut self, v: &mut Editor, mouse_info: MouseInfo) {
        if !v.is_modifying() {
            v.begin_modification(&self.describe_history_entry(), false);
            self.pressed_pos = Some((mouse_info.position.0, mouse_info.position.1));
            self.locked_angle = mouse_info.modifiers.ctrl;
        }
    }

    fn mouse_moved(&mut self, v: &mut Editor, mouse_info: MouseInfo) {
        if let Some(pos) = self.pressed_pos {
            self.locked_angle = mouse_info.modifiers.ctrl;
            if self.dropped_shape {
                let layer = v.get_active_layer_mut();
                if layer.outline.len() > 0 {
                    layer.outline.remove(layer.outline.len() - 1);
                }
            }

            let mut sd = ShapeDrawer {
                mouse_info,
                from: pos,
                sdata: self.sdata,
                corners: None,
            };
            let o = match self.stype {
                ShapeType::Circle => sd.draw_circle(),
                ShapeType::Oval | ShapeType::Rectangle | ShapeType::RoundedRectangle => {
                    sd.draw_fits_in_rect(self.stype)
                }
                ShapeType::Polygon | ShapeType::Star => {
                    if !self.locked_angle {
                        self.sdata.polygon_angle = sd.polygon_angle();
                    }
                    sd.draw_polygon(self.stype)
                }
            };
            self.corners = sd.corners;

            let mfek_o: Vec<MFEKContour<MFEKPointData>> =
                o.iter().map(|e| e.into()).collect();
            v.get_active_layer_mut().outline.extend(mfek_o);
            self.dropped_shape = true;
        }
    }

    fn mouse_released(&mut self, v: &mut Editor, _mouse_info: MouseInfo) {
        v.redescribe_modification(self.describe_history_entry());
        v.end_modification();
        self.pressed_pos = None;
        self.dropped_shape = false;
        self.corners = None;
    }

    fn scroll(&mut self, vertical: i32) {
        let prev = vertical > 0;
        for _ in 0..vertical.abs() {
            if prev {
                self.stype = self.stype.prev();
            } else {
                self.stype = self.stype.next();
            }
        }
    }
}
