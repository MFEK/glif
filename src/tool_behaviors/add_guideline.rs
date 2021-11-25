use glifparser::{Guideline, IntegerOrFloat::Float};
use glifrenderer::guidelines::draw_guideline;

use super::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct AddGuideline {
    angle: f32,
}

impl AddGuideline {
    pub fn new(angle: f32) -> Self {
        AddGuideline { angle }
    }

    pub fn mouse_released(&mut self, v: &mut Editor, mouse_info: MouseInfo) {
        let prev_guide = Guideline::from_x_y_angle(calc_x(mouse_info.position.0), calc_y(mouse_info.position.1), Float(self.angle));

        v.begin_modification("Add guideline.");
        v.with_glyph_mut(|glif| {
            glif.guidelines.push(prev_guide.clone());
        });
        v.end_modification();

        v.pop_behavior();
    }
}

impl ToolBehavior for AddGuideline {
    fn event(&mut self, v: &mut Editor, _i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent {
                event_type: MouseEventType::Released,
                mouse_info,
            } => {
                self.mouse_released(v, mouse_info);
            }
            _ => (),
        }
    }

    fn draw(&self, _v: &Editor, i: &Interface, canvas: &mut Canvas) {
        let prev_guide = Guideline::<()>::from_x_y_angle(calc_x(i.mouse_info.position.0), calc_y(i.mouse_info.position.1), Float(self.angle));
        draw_guideline(&i.viewport, canvas, &prev_guide, None)
    }
}
