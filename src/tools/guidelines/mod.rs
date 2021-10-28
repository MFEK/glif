use glifrenderer::guidelines::draw_guideline;

use super::prelude::*;
use crate::editor::Editor;
use crate::tool_behaviors::move_guideline::MoveGuideline;
use crate::user_interface::Interface;

mod dialog;

#[derive(Clone)]
pub struct Guidelines {
    selected_idx: Option<usize>,
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
        }
    }

    fn mouse_pressed(&mut self, v: &mut Editor, i: &mut Interface, mouse_info: MouseInfo) {
        self.selected_idx = None;

        v.with_glyph(|glyph| {
            for (idx, guide) in glyph.guidelines.iter().enumerate() {
                let angle = f32::from(guide.angle);
                let position = i.mouse_info.position.clone();

                // x offset from mouse_pos to the guideline's pos
                let dx = guide.at.x - position.0;

                // now we calculate where the the y coordinate of the line should be at the mouse's xpos
                let pos_y = guide.at.y - dx * angle.to_radians().tan();

                if (pos_y - calc_y(position.1)).abs() < 5. / i.viewport.factor {
                    self.selected_idx = Some(idx);
                    return;
                }
            }

            self.selected_idx = None;
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
