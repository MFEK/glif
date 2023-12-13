use super::prelude::*;
use glifrenderer::constants::{OUTLINE_STROKE, OUTLINE_STROKE_THICKNESS};

#[derive(Clone, Debug)]
pub struct DrawPivot {
    pub command_mod: Option<CommandMod>,
    pub button: MouseButton,
    pub pivot_point: Option<(f32, f32)>,
}

impl Default for DrawPivot {
    fn default() -> Self {
        Self {
            command_mod: None,
            button: MouseButton::Right,
            pivot_point: None,
        }
    }
}

impl ToolBehavior for DrawPivot {
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

    fn draw(&mut self, v: &Editor, i: &Interface, canvas: &Canvas) {
        self.draw_pivot_point(v, i, canvas);
    }
}

impl DrawPivot {
    fn mouse_pressed(&mut self, _v: &Editor, _i: &Interface, mouse_info: MouseInfo) {
        if let Some(cm) = self.command_mod {
            if cm != mouse_info.modifiers {
                return;
            }
        }

        if self.button != mouse_info.button {
            return;
        }

        self.pivot_point = Some(mouse_info.position);
    }

    fn draw_pivot_point(&self, _v: &Editor, i: &Interface, canvas: &Canvas) {
        if let Some(pivot) = self.pivot_point {
            let pivot = (pivot.0, pivot.1);
            let mut paint = Paint::default();

            paint.set_anti_alias(true);
            paint.set_color(OUTLINE_STROKE);
            paint.set_style(PaintStyle::Stroke);
            paint.set_stroke_width(OUTLINE_STROKE_THICKNESS * (1. / i.viewport.factor));
            canvas.draw_circle(pivot, 5. * (1. / i.viewport.factor), &paint);
        }
    }
}
