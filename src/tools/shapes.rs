use std::f32::consts::PI;

use super::prelude::*;

use crate::user_interface::Interface;

use glifparser::{glif::MFEKContour, outline::skia::FromSkiaPath, Outline};
use imgui;
use skulpin::skia_safe::{
    Matrix, Path, PathDirection, PathEffect, Point as SkPoint, RRect, Rect, StrokeRec,
};

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
            _ => {}
        }
    }

    fn ui(&mut self, _v: &mut Editor, i: &mut Interface, ui: &mut Ui) {
        self.shape_settings(i, ui);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShapeType {
    Circle,
    Oval,
    Rectangle,
    RoundedRectangle,
    Polygon,
}

#[derive(Clone, Debug)]
pub struct Shapes {
    pressed_pos: Option<(f32, f32)>,
    dropped_shape: bool,
    // Because of imgui, we can't have associated types on the ShapeType enum. Thus, this
    // compromise.
    stype: ShapeType,
    sdata: ShapeData,
}

#[derive(Copy, Clone, Debug)]
pub struct ShapeData {
    polygon_sides: u16,
    polygon_evenodd: bool,
    polygon_radius: f32,
    rrect_radius: f32,
}

impl Default for ShapeData {
    fn default() -> Self {
        Self {
            polygon_sides: 5,
            polygon_evenodd: false,
            rrect_radius: 50.,
            polygon_radius: 0.,
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
        }
    }

    fn shape_settings(&mut self, i: &mut Interface, ui: &imgui::Ui) {
        let (tx, ty, tw, th) = i.get_tools_dialog_rect();
        imgui::Window::new(imgui::im_str!("Shape Settings"))
            .bg_alpha(1.) // See comment on fn redraw_skia
            .flags(
                imgui::WindowFlags::NO_RESIZE
                    | imgui::WindowFlags::NO_MOVE
                    | imgui::WindowFlags::NO_COLLAPSE,
            )
            .position([tx, ty], imgui::Condition::Always)
            .size([tw, th], imgui::Condition::Always)
            .build(ui, || {
                ui.radio_button(imgui::im_str!("Circle"), &mut self.stype, ShapeType::Circle);
                ui.radio_button(imgui::im_str!("Oval"), &mut self.stype, ShapeType::Oval);
                ui.radio_button(
                    imgui::im_str!("Rectangle"),
                    &mut self.stype,
                    ShapeType::Rectangle,
                );
                ui.radio_button(
                    imgui::im_str!("Rounded Rectangle"),
                    &mut self.stype,
                    ShapeType::RoundedRectangle,
                );
                ui.radio_button(
                    imgui::im_str!("Polygon (Star)"),
                    &mut self.stype,
                    ShapeType::Polygon,
                );

                match self.stype {
                    ShapeType::RoundedRectangle => {
                        imgui::Slider::new(imgui::im_str!("Roundness"))
                            .range(1f32..=1000f32)
                            .build(ui, &mut self.sdata.rrect_radius);
                    }
                    ShapeType::Polygon => {
                        imgui::Slider::new(imgui::im_str!("Sides"))
                            .range(3u16..=50u16)
                            .build(ui, &mut self.sdata.polygon_sides);
                        imgui::Slider::new(imgui::im_str!("Roundness"))
                            .range(1f32..=1000f32)
                            .build(ui, &mut self.sdata.polygon_radius);
                        ui.checkbox(imgui::im_str!("Star?"), &mut self.sdata.polygon_evenodd);
                    }
                    _ => (),
                }
            });
    }
}

struct ShapeDrawer {
    mouse_info: MouseInfo,
    from: (f32, f32),
    sdata: ShapeData,
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
            self.from.0 - calc_x(self.mouse_info.position.0),
            self.from.1 - calc_y(self.mouse_info.position.1),
        );
        (cx, cy, ((cx).powf(2.) + (cy).powf(2.)).sqrt())
    }

    fn draw_circle(&self) -> Outline<MFEKPointData> {
        let (cx, _cy, dist) = self.calculate_radius();
        let path = Path::circle(self.from, dist, Some(shape_direction(cx)));
        Outline::from_skia_path(&path)
    }

    // Odd even causes a pentagram at 5, and interesting connected shapes at odd numbers above 5.
    fn draw_polygon(&self, polygon: ShapeType) -> Outline<MFEKPointData> {
        let (sides, oddeven) = if polygon == ShapeType::Polygon {
            (self.sdata.polygon_sides, self.sdata.polygon_evenodd)
        } else {
            panic!("Called draw_polygon without a ShapeType::Polygon!");
        };

        let (cxm, cym, radius) = self.calculate_radius();
        let rotangle = (cym / cxm).atan();
        let rotangle = rotangle * (180. / PI);
        let rotangle = if cxm > 0. { 180. + rotangle } else { rotangle };
        // I'm not sure if this is a Skia thing, or a "Fred is bad at math" thing, but this does
        // fix it.
        let rotangle = if rotangle == 90. || rotangle == -90. {
            -rotangle
        } else {
            rotangle
        };
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
            let pathbuild_func = if i == 0 { Path::move_to } else { Path::line_to };
            // It's pretty cool that this works, isn't it? Rust pretending to be Python again.
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

    fn draw_fits_in_rect(&self, stype: ShapeType) -> Outline<MFEKPointData> {
        let rect = Rect::new(
            self.from.0,
            self.from.1,
            calc_x(self.mouse_info.position.0),
            calc_y(self.mouse_info.position.1),
        );
        let (cx, _, _) = self.calculate_radius();
        let path = match stype {
            ShapeType::Oval => Path::oval(rect, Some(shape_direction(-cx))),
            ShapeType::RoundedRectangle => {
                let rrect = RRect::new_rect_radii(
                    rect,
                    &[SkPoint::new(self.sdata.rrect_radius, self.sdata.rrect_radius); 4],
                );
                Path::rrect(rrect, Some(shape_direction(-cx)))
            }
            ShapeType::Rectangle => Path::rect(rect, Some(shape_direction(-cx))),
            _ => unimplemented!("ShapeType doesn't fit in rect, or unimplemented"),
        };
        Outline::from_skia_path(&path)
    }
}

impl Shapes {
    fn mouse_pressed(&mut self, v: &mut Editor, mouse_info: MouseInfo) {
        if !v.is_modifying() {
            v.begin_modification("Draw shape");
            self.pressed_pos = Some((calc_x(mouse_info.position.0), calc_y(mouse_info.position.1)));
        }
    }

    fn mouse_moved(&mut self, v: &mut Editor, mouse_info: MouseInfo) {
        if let Some(pos) = self.pressed_pos {
            if self.dropped_shape {
                v.with_active_layer_mut(|layer| {
                    layer.outline.remove(layer.outline.len() - 1);
                });
            }

            v.with_active_layer_mut(|layer| {
                let sd = ShapeDrawer {
                    mouse_info,
                    from: pos,
                    sdata: self.sdata,
                };
                let o = match self.stype {
                    ShapeType::Circle => sd.draw_circle(),
                    ShapeType::Oval | ShapeType::Rectangle | ShapeType::RoundedRectangle => {
                        sd.draw_fits_in_rect(self.stype)
                    }
                    ShapeType::Polygon => sd.draw_polygon(self.stype),
                };

                let mfek_o: Vec<MFEKContour<MFEKPointData>> = o.iter().map(|e| e.into()).collect();
                layer.outline.extend(mfek_o);
                self.dropped_shape = true;
            });
        }
    }

    fn mouse_released(&mut self, v: &mut Editor, _mouse_info: MouseInfo) {
        v.end_modification();
        self.pressed_pos = None;
        self.dropped_shape = false;
    }
}
