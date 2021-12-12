use super::prelude::*;

use crate::command::{Command::{*, self}, CommandType};

#[derive(Clone, Debug)]
pub struct PanBehavior {
    viewport: Viewport,
    mouse_info: MouseInfo,
}

impl PanBehavior {
    pub fn new(viewport: Viewport, mouse_info: MouseInfo) -> Self {
        PanBehavior {
            viewport,
            mouse_info,
        }
    }

    pub fn mouse_moved(&mut self, _v: &mut Editor, i: &mut Interface, mouse_info: MouseInfo) {
        let mut new_offset = self.viewport.offset;
        new_offset.0 +=
            (mouse_info.absolute_position.0 - self.mouse_info.absolute_position.0).floor();
        new_offset.1 +=
            (mouse_info.absolute_position.1 - self.mouse_info.absolute_position.1).floor();

        i.viewport.offset = new_offset;
    }

    pub fn mouse_released(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        if mouse_info.button == self.mouse_info.button {
            v.pop_behavior();
        }
    }
}

impl PanBehavior {
    pub fn nudge_factor(command: Command) -> f32 {
        use crate::constants::{BIG_OFFSET_FACTOR, OFFSET_FACTOR, TINY_OFFSET_FACTOR};
        match command {
            NudgeTinyUp | NudgeTinyDown | NudgeTinyLeft | NudgeTinyRight => TINY_OFFSET_FACTOR,
            NudgeBigUp | NudgeBigDown | NudgeBigLeft | NudgeBigRight => BIG_OFFSET_FACTOR,
            NudgeUp | NudgeDown | NudgeLeft | NudgeRight => OFFSET_FACTOR,
            _ => f32::NAN,
        }
    }
    pub fn nudge_offset(command: Command, factor: f32) -> (f32, f32) {
        match command {
            NudgeUp | NudgeTinyUp | NudgeBigUp => (0., factor),
            NudgeDown | NudgeTinyDown | NudgeBigDown => (0., -factor),
            NudgeLeft | NudgeTinyLeft | NudgeBigLeft => (factor, 0.),
            NudgeRight | NudgeTinyRight | NudgeBigRight => (-factor, 0.),
            _ => (f32::NAN, f32::NAN),
        }
    }
}

#[rustfmt::skip]
impl ToolBehavior for PanBehavior {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent { event_type, mouse_info } => {
                match event_type {
                    MouseEventType::Released => self.mouse_released(v, i, mouse_info),
                    MouseEventType::Moved => self.mouse_moved(v, i, mouse_info),
                    _ => {}
                }
            },
            EditorEvent::ToolCommand {
                command,
                stop_after,
                ..
            } => {
                if command.type_() == CommandType::Nudge {
                    *stop_after = true;
                    let factor = PanBehavior::nudge_factor(command);
                    let offset = PanBehavior::nudge_offset(command, factor);
                    i.nudge_viewport(offset);
                }
            }
        }
    }
}
