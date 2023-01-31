use glifparser::{Guideline, IntegerOrFloat::Float};
use glifrenderer::guidelines::draw_guideline;

use super::prelude::*;

use crate::command::Command;

#[derive(Clone, Debug, Default)]
pub struct AddGuideline {
    angle: f32,
    global: bool,
    new_angle_idx: usize,
}

impl AddGuideline {
    pub fn new(angle: f32, global: bool) -> Self {
        AddGuideline {
            angle,
            global,
            new_angle_idx: 0,
        }
    }

    pub fn mouse_released(&mut self, v: &mut Editor, mouse_info: MouseInfo) {
        let prev_guide = Guideline::from_x_y_angle(
            mouse_info.position.0,
            mouse_info.position.1,
            Float(self.angle),
        );

        v.begin_modification("Add guideline.", false);
        if self.global {
            v.guidelines.push(prev_guide.clone());
        } else {
            v.with_glyph_mut(|glif| {
                glif.guidelines.push(prev_guide.clone());
            });
        }
        v.mark_dirty();
        v.end_modification();
        v.pop_behavior();
    }
    fn next_angle_type(&mut self, v: &Editor) {
        let angle_types = if v.italic_angle == 0. {
            vec![0., 90.]
        } else {
            vec![0., v.italic_angle, 90.]
        };
        self.new_angle_idx = if self.new_angle_idx == angle_types.len() - 1 {
            0
        } else {
            self.new_angle_idx + 1
        };
        self.angle = angle_types[self.new_angle_idx];
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
            EditorEvent::ToolCommand {
                command: Command::ReverseContour,
                stop_after,
                ..
            } => {
                self.next_angle_type(v); // usually the same as a "reversal"
                *stop_after.borrow_mut() = true;
            }
            _ => (),
        }
    }

    fn draw(&mut self, _v: &Editor, i: &Interface, canvas: &mut Canvas) {
        let prev_guide = Guideline::<()>::from_x_y_angle(
            i.mouse_info.position.0,
            i.mouse_info.position.1,
            Float(self.angle),
        );
        draw_guideline(&i.viewport, canvas, &prev_guide, None)
    }
}
