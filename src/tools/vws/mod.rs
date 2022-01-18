mod dialog;
pub mod util;

use crate::tool_behaviors::{move_vws_handle::MoveVWSHandle, zoom_scroll::ZoomScroll};
use crate::user_interface::Interface;

use glifparser::glif::ContourOperations;
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
        match event {
            EditorEvent::MouseEvent { mouse_info, event_type } => match event_type {
                MouseEventType::Pressed => self.mouse_pressed(v, i, mouse_info),
                _ => (),
            }
            EditorEvent::ScrollEvent { .. } => ZoomScroll::default().event(v, i, event),
            _ => {}
        }
    }

    fn draw(&mut self, v: &Editor, i: &Interface, canvas: &mut Canvas) {
        self.draw_handles(v, i, canvas);
    }

    fn ui(&mut self, v: &mut Editor, i: &mut Interface, ui: &mut Ui) {
        let show_dialog = v.with_active_layer(|layer| match v.contour_idx {
            Some(ci) => match layer.outline[ci].operation {
                Some(ContourOperations::VariableWidthStroke { .. }) => true,
                _ => false,
            },
            _ => false,
        });
        if show_dialog {
            self.tool_dialog(v, i, ui);
        }
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

                    let (handle_pos_left, handle_pos_right) = match (
                        get_vws_handle_pos(v, contour_idx, vws_handle_idx, WhichHandle::A),
                        get_vws_handle_pos(v, contour_idx, vws_handle_idx, WhichHandle::B),
                    ) {
                        (Ok(l), Ok(r)) => (l.2, r.2),
                        (_e_l, _e_r) => continue,
                    };

                    let mut path = SkiaPath::new();
                    let mut paint = Paint::default();

                    paint.set_anti_alias(true);
                    paint.set_color(RIB_STROKE);
                    paint.set_stroke_width(HANDLEBAR_THICKNESS * (1. / factor));
                    paint.set_style(PaintStyle::Stroke);

                    path.move_to((handle_pos_left.x as f32, handle_pos_left.y as f32));
                    path.line_to((start_point.x as f32, start_point.y as f32));
                    path.line_to((handle_pos_right.x as f32, handle_pos_right.y as f32));

                    canvas.draw_path(&path, &paint);
                }

                if contour.inner.first().unwrap().ptype == glifparser::PointType::Move {
                    if contour_pw.segs.len() < 1 {
                        continue;
                    };
                    let vws_handle_idx = contour_pw.segs.len();
                    let bezier = contour_pw.segs.last().unwrap();
                    let start_point = bezier.end_point();

                    let (handle_pos_left, handle_pos_right) = match (
                        get_vws_handle_pos(v, contour_idx, vws_handle_idx, WhichHandle::A),
                        get_vws_handle_pos(v, contour_idx, vws_handle_idx, WhichHandle::B),
                    ) {
                        (Ok(l), Ok(r)) => (l.2, r.2),
                        (_e_l, _e_r) => continue,
                    };

                    let mut path = SkiaPath::new();
                    let mut paint = Paint::default();

                    paint.set_anti_alias(true);
                    paint.set_color(RIB_STROKE);
                    paint.set_stroke_width(HANDLEBAR_THICKNESS * (1. / factor));
                    paint.set_style(PaintStyle::Stroke);

                    path.move_to((handle_pos_left.x as f32, handle_pos_left.y as f32));
                    path.line_to((start_point.x as f32, start_point.y as f32));
                    path.line_to((handle_pos_right.x as f32, handle_pos_right.y as f32));

                    canvas.draw_path(&path, &paint);
                }
            }
        })
    }
}
