mod dialog;
pub mod util;

use crate::tool_behaviors::{move_vws_handle::MoveVWSHandle, zoom_scroll::ZoomScroll};
use crate::user_interface::Interface;

use glifparser::glif::contour::MFEKContourCommon;
use glifparser::glif::contour_operations::ContourOperations;
use sdl2::mouse::MouseButton;
use skulpin::skia_safe::{Canvas, Paint, PaintStyle, Path as SkiaPath};

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
        let show_dialog = match v.contour_idx {
            Some(ci) => match v.get_active_layer_ref().outline[ci].operation {
                Some(ContourOperations::VariableWidthStroke { .. }) => true,
                _ => false,
            },
            _ => false,
        };
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
        let factor = i.viewport.factor;

        for (contour_idx, contour) in v.get_active_layer_ref().outline.iter().enumerate() {
            for (vws_handle_idx, point) in contour.iter().enumerate() {

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
                path.line_to((point.x(), point.y()));
                path.line_to((handle_pos_right.x as f32, handle_pos_right.y as f32));

                canvas.draw_path(&path, &paint);
            }

            if contour.is_open() {
                if contour.len() < 1 {
                    continue;
                };
                let vws_handle_idx = contour.len() - 1;
                let point = contour.get_point(contour.len()-1).unwrap();

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
                path.line_to((point.x(), point.y()));
                path.line_to((handle_pos_right.x as f32, handle_pos_right.y as f32));

                canvas.draw_path(&path, &paint);
            }
        }
    }
}
