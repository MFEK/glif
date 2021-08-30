use flo_curves::bezier::solve_curve_for_t;
use MFEKmath::{Bezier, Vector};
use glifrenderer::guidelines::draw_guideline;

use super::prelude::*;
use crate::editor::Editor;
use crate::tool_behaviors::move_guideline::MoveGuideline;
use crate::user_interface::Interface;

mod dialog;

#[derive(Clone)]
pub struct Guidelines {
    selected_idx: Option<usize>,
    move_position: Option<(f32, f32)>,
}

impl Tool for Guidelines {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent {
                event_type,
                mouse_info,
            } => match event_type {
                MouseEventType::Pressed => self.mouse_pressed(v, i, mouse_info),
                _ => {}
            },
            _ => {}
        }
    }

    fn draw(&self, v: &Editor, i: &Interface, canvas: &mut Canvas) {
        self.draw_selected_guideline(i, v, canvas);
    }

    fn ui(&mut self, v: &mut Editor, i: &mut Interface, ui: &mut Ui) {
        self.tool_dialog(v, i, ui)
    }
}

impl Guidelines {
    pub fn new() -> Self {
        Self {
            selected_idx: None,
            move_position: None,
        }
    }

    fn mouse_pressed(&mut self, v: &mut Editor, i: &mut Interface, mouse_info: MouseInfo) {
        self.selected_idx = None;

        let vp = &i.viewport;
        v.with_glyph(|glyph| {
            for (idx, guide) in glyph.guidelines.iter().enumerate() {
                let angle = guide.angle;
                let at2 = Vector::from_components(
                    (guide.at.x + ((1000. * vp.winsize.0 as f32) * f32::from(angle).cos())) as f64,
                    (guide.at.y + ((1000. * vp.winsize.1 as f32) * f32::from(angle).sin())) as f64,
                );
                let at3 = Vector::from_components(
                    (guide.at.x + ((-(1000. * vp.winsize.0 as f32)) * f32::from(angle).cos()))
                        as f64,
                    (guide.at.y + ((-(1000. * vp.winsize.1 as f32)) * f32::from(angle).sin()))
                        as f64,
                );

                let position = i.mouse_info.position.clone();
                let mouse_vec = Vector::from_components(
                    calc_x(position.0) as f64,
                    calc_y(position.1 as f32) as f64,
                );
                let line_bez = Bezier::from_points(at2, at2, at3, at3);

                if let Some(_point_info) =
                    solve_curve_for_t(&line_bez, &mouse_vec, 100000. / i.viewport.factor as f64)
                {
                    self.selected_idx = Some(idx);
                }
            }
        });

        if let Some(selected) = self.selected_idx {
            v.set_behavior(Box::new(MoveGuideline::new(selected, mouse_info)));
        }
    }

    fn draw_selected_guideline(&self, i: &Interface, v: &Editor, canvas: &mut Canvas) {
        if let Some(selected) = self.selected_idx {
            v.with_glyph(|glif| {
                draw_guideline(
                    &i.viewport,
                    canvas,
                    &glif.guidelines[selected],
                    Some(SELECTED_FILL),
                )
            })
        }
    }
}
