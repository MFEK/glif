use sdl2::mouse::MouseButton;

use crate::command::CommandMod;

use super::Editor;

/// This struct stores the editor's mouse state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseInfo {
    pub button: MouseButton,
    pub position: (f32, f32),
    pub absolute_position: (f32, f32),
    pub is_down: bool,
    pub modifiers: CommandMod,
}

impl Default for MouseInfo {
    fn default() -> Self { 
        MouseInfo {
            button: sdl2::mouse::MouseButton::Unknown,
            position: (0., 0.),
            absolute_position: (0., 0.),
            is_down: false,
            modifiers: CommandMod::none(),
        }
    }
}

impl MouseInfo {
    pub fn new(
        v: &Editor,
        button: Option<MouseButton>,
        position: (f32, f32),
        mousedown: Option<bool>,
        command_mod: CommandMod,
    ) -> MouseInfo {
        let factor = 1. / v.viewport.factor;
        let uoffset = v.viewport.offset;
        let offset = (uoffset.0, uoffset.1);
    
        let absolute_mposition = ((position.0).floor(), (position.1).floor());
        let mposition = (
            ((position.0).floor() - offset.0) * factor,
            ((position.1).floor() - offset.1) * factor,
        );

        MouseInfo {
            button: button.unwrap_or(v.mouse_info.button),
            is_down: mousedown.unwrap_or(v.mouse_info.is_down),
            modifiers: command_mod,
            position: mposition,
            absolute_position: absolute_mposition,
        }
    }
}

impl Editor {
        // Generic events
    pub fn center_cursor(&mut self) {
        let mut center = self.sdl_window.as_ref().unwrap().drawable_size();
        center.0 /= 2;
        center.1 /= 2;
        self.mouse_info.absolute_position = (center.0 as f32, center.1 as f32);

        self.sdl_context
            .as_ref()
            .unwrap()
            .mouse()
            .warp_mouse_in_window(&self.sdl_window.as_ref().unwrap(), center.0 as i32, center.1 as i32);
    }

}
