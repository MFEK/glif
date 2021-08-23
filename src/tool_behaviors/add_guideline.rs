use glifparser::{Guideline, GuidelinePoint, IntegerOrFloat};

use crate::renderer::guidelines::draw_guideline;

use super::prelude::*;

#[derive(Clone)]
pub struct AddGuideline {}

impl AddGuideline {
    pub fn new() -> Self {
        Self {}
    }

    pub fn mouse_released(&mut self, v: &mut Editor, i: &mut Interface, mouse_info: MouseInfo) {
        let abs_pos = mouse_info.absolute_position;
        let top = 0.;
        let left = 0.;
        let bottom = i.viewport.winsize.1 as f32;
        let right = i.viewport.winsize.0 as f32;

        let min_distance_vertical =
            f32::min(f32::abs(abs_pos.1 - bottom), f32::abs(abs_pos.1 - top));
        let min_distance_horizontal =
            f32::min(f32::abs(abs_pos.0 - left), f32::abs(abs_pos.0 - right));

        if min_distance_vertical < min_distance_horizontal {
            let prev_point = GuidelinePoint {
                x: i.mouse_info.position.0,
                y: calc_y(i.mouse_info.position.1),
            };

            let prev_guide = Guideline {
                at: prev_point,
                angle: IntegerOrFloat::Float(90.),
                name: None,
                color: None,
                identifier: None,
            };

            v.with_glyph_mut(|glif| {
                glif.guidelines.push(prev_guide.clone());
            });
        } else {
            let prev_point = GuidelinePoint {
                x: i.mouse_info.position.0,
                y: calc_y(i.mouse_info.position.1),
            };

            let prev_guide = Guideline {
                at: prev_point,
                angle: IntegerOrFloat::Float(0.),
                name: None,
                color: None,
                identifier: None,
            };

            v.with_glyph_mut(|glif| {
                glif.guidelines.push(prev_guide.clone());
            });
        }

        v.pop_behavior();
    }
}

impl ToolBehavior for AddGuideline {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent {
                event_type,
                mouse_info,
            } => match event_type {
                MouseEventType::Released => self.mouse_released(v, i, mouse_info),
                _ => {}
            },
            _ => {}
        }
    }

    fn draw(&self, _v: &Editor, i: &Interface, canvas: &mut Canvas) {
        let abs_pos = i.mouse_info.absolute_position;
        let top = 0.;
        let left = 0.;
        let bottom = i.viewport.winsize.1 as f32;
        let right = i.viewport.winsize.0 as f32;

        let min_distance_vertical =
            f32::min(f32::abs(abs_pos.1 - bottom), f32::abs(abs_pos.1 - top));
        let min_distance_horizontal =
            f32::min(f32::abs(abs_pos.0 - left), f32::abs(abs_pos.0 - right));

        if min_distance_vertical < min_distance_horizontal {
            let prev_point = GuidelinePoint {
                x: i.mouse_info.position.0,
                y: calc_y(i.mouse_info.position.1),
            };

            let prev_guide = Guideline {
                at: prev_point,
                angle: IntegerOrFloat::Float(90.),
                name: None,
                color: None,
                identifier: None,
            };

            draw_guideline(&i.viewport, canvas, &prev_guide, None)
        } else {
            let prev_point = GuidelinePoint {
                x: i.mouse_info.position.0,
                y: calc_y(i.mouse_info.position.1),
            };

            let prev_guide = Guideline {
                at: prev_point,
                angle: IntegerOrFloat::Float(0.),
                name: None,
                color: None,
                identifier: None,
            };

            draw_guideline(&i.viewport, canvas, &prev_guide, None)
        }
    }
}
