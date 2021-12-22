use crate::command::{Command, CommandMod};
use crate::user_interface::MouseInfo;

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEventType {
    Pressed,
    DoubleClick,
    Released,
    Moved,
    Scrolled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IOEventType {
    FileSwitched,
    FileSaved,
    FileSavedAs,
    FileFlattened,
    FileFlattenedAs,
    FileExported,
    FontinfoWritten,
    /// This will always trigger after a FontinfoWritten, but FontinfoWritten won't trigger unless
    /// *we* rewrote the Fontinfo.
    FontinfoReloaded,
}

#[derive(Debug)]
pub enum EditorEvent<'frame> {
    MouseEvent {
        event_type: MouseEventType,
        mouse_info: MouseInfo,
    },

    ScrollEvent {
        /// Assumed to almost always be unused, few devices support this
        horizontal: i32,
        /// Main scroll wheel use
        /// From SDL docs: positive away from the user and negative towards the user
        vertical: i32,
    },

    /// I/O (file input/output) events
    IOEvent {
        event_type: IOEventType,
        path: PathBuf,
    },

    ToolCommand {
        command: Command,
        command_mod: CommandMod,
        stop_after: &'frame mut bool,
    },
}
