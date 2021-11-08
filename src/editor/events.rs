use crate::command::{Command, CommandMod};
use crate::user_interface::MouseInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEventType {
    Pressed,
    DoubleClick,
    Released,
    Moved,
}

#[derive(Debug)]
pub enum EditorEvent<'a> {
    MouseEvent {
        event_type: MouseEventType,
        mouse_info: MouseInfo,
    },

    ToolCommand {
        command: Command,
        command_mod: CommandMod,
        stop_after: &'a mut bool,
    },
}
