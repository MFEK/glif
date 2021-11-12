mod dialog;
pub mod util;

use crate::{tool_behaviors::move_vws_handle::MoveVWSHandle, user_interface::Interface};
use sdl2::mouse::MouseButton;
use skulpin::skia_safe::{Canvas, Paint, PaintStyle, Path as SkiaPath};
use MFEKmath::{Evaluate, Piecewise};

use self::util::{clicked_handle, get_vws_handle_pos};

use super::prelude::*;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug)]
pub struct VWS {}

impl Tool for VWS {
    #[rustfmt::skip]
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        if let EditorEvent::MouseEvent { mouse_info, event_type } = event {
            match event_type {
                MouseEventType::Pressed => self.mouse_pressed(v, i, mouse_info),
                _ => (),
            }
        }
    }

    fn draw(&self, v: &Editor, i: &Interface, canvas: &mut Canvas) {
        self.draw_handles(v, i, canvas);
    }

    fn ui(&mut self, v: &mut Editor, i: &mut Interface, ui: &mut Ui) {
        self.build_vws_settings_window(v, i, ui)
    }
}

impl VWS {
    pub fn new() -> Self {
        VWS {}
    }

    fn mouse_pressed(&mut self, v: &mut Editor, i: &Interface, mouse_info: MouseInfo) {
        if let Some((ci, pi, wh)) = clicked_handle(v, i, mouse_info) {
            v.contour_idx = Some(ci);
            v.point_idx = Some(pi);
            v.selected.clear();

            v.set_behavior(Box::new(MoveVWSHandle::new(
                mouse_info.button == MouseButton::Left,
                mouse_info.modifiers.ctrl,
                mouse_info.modifiers.shift,
                wh,
                mouse_info,
            )));
        }
    }

    pub fn draw_handles(&self, v: &Editor, i: &Interface, canvas: &mut Canvas) {
        v.with_active_layer(|layer| {
            let factor = i.viewport.factor;

            for (contour_idx, contour) in layer.outline.iter().enumerate() {
                let contour_pw = Piecewise::from(contour);

                for (vws_handle_idx, bezier) in contour_pw.segs.iter().enumerate() {
                    let start_point = bezier.start_point();
                    let handle_pos_left =
                        get_vws_handle_pos(v, contour_idx, vws_handle_idx, WhichHandle::A).2;
                    let handle_pos_right =
                        get_vws_handle_pos(v, contour_idx, vws_handle_idx, WhichHandle::B).2;

                    let mut path = SkiaPath::new();
                    let mut paint = Paint::default();

                    paint.set_anti_alias(true);
                    paint.set_color(RIB_STROKE);
                    paint.set_stroke_width(HANDLEBAR_THICKNESS * (1. / factor));
                    paint.set_style(PaintStyle::Stroke);

                    path.move_to((
                        calc_x(handle_pos_left.x as f32),
                        calc_y(handle_pos_left.y as f32),
                    ));
                    path.line_to((calc_x(start_point.x as f32), calc_y(start_point.y as f32)));
                    path.line_to((
                        calc_x(handle_pos_right.x as f32),
                        calc_y(handle_pos_right.y as f32),
                    ));

                    canvas.draw_path(&path, &paint);
                }

                if contour.inner.first().unwrap().ptype == glifparser::PointType::Move {
                    if contour_pw.segs.len() < 2 {
                        continue;
                    };
                    let vws_handle_idx = contour_pw.segs.len();
                    let bezier = contour_pw.segs.last().unwrap();
                    let start_point = bezier.end_point();

                    let handle_pos_left =
                        get_vws_handle_pos(v, contour_idx, vws_handle_idx, WhichHandle::A).2;
                    let handle_pos_right =
                        get_vws_handle_pos(v, contour_idx, vws_handle_idx, WhichHandle::B).2;

                    let mut path = SkiaPath::new();
                    let mut paint = Paint::default();

                    paint.set_anti_alias(true);
                    paint.set_color(RIB_STROKE);
                    paint.set_stroke_width(HANDLEBAR_THICKNESS * (1. / factor));
                    paint.set_style(PaintStyle::Stroke);

                    path.move_to((
                        calc_x(handle_pos_left.x as f32),
                        calc_y(handle_pos_left.y as f32),
                    ));
                    path.line_to((calc_x(start_point.x as f32), calc_y(start_point.y as f32)));
                    path.line_to((
                        calc_x(handle_pos_right.x as f32),
                        calc_y(handle_pos_right.y as f32),
                    ));

                    canvas.draw_path(&path, &paint);
                }
            }
        })
    }
}
