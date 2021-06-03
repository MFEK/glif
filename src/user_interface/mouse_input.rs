use sdl2::mouse::MouseButton;

use crate::{command::CommandMod, user_interface::Interface};

/// This struct stores the editor's mouse state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseInfo {
    pub button: MouseButton,

    pub position: (f32, f32),
    pub raw_position: (f32, f32),
    pub absolute_position: (f32, f32),
    pub raw_absolute_position: (f32, f32),

    pub is_down: bool,
    pub modifiers: CommandMod,
}

impl Default for MouseInfo {
    fn default() -> Self { 
        MouseInfo {
            button: sdl2::mouse::MouseButton::Unknown,
            position: (0., 0.),
            raw_position: (0., 0.),
            absolute_position: (0., 0.),
            raw_absolute_position: (0., 0.),
            is_down: false,
            modifiers: CommandMod::none(),
        }
    }
}

impl MouseInfo {
    pub fn new(
        i: &Interface,
        button: Option<MouseButton>,
        position: (f32, f32),
        mousedown: Option<bool>,
        command_mod: CommandMod,
    ) -> MouseInfo {
        let factor = 1. / i.viewport.factor;
        let uoffset = i.viewport.offset;
        let offset = (uoffset.0, uoffset.1);
    
        let mut absolute_mposition = ((position.0).floor(), (position.1).floor());
        let mut mposition = (
            ((position.0).floor() - offset.0) * factor,
            ((position.1).floor() - offset.1) * factor,
        );

        let raw_absolute_mposition = absolute_mposition;
        let raw_mposition = mposition;

        if let Some(grid) = &i.grid {
            absolute_mposition.0 = (absolute_mposition.0 / grid.spacing).round() * grid.spacing;
            absolute_mposition.1 = (absolute_mposition.1 / grid.spacing).round() * grid.spacing;

            mposition = (
                (mposition.0 / grid.spacing).round() * grid.spacing,
                (mposition.1 / grid.spacing).round() * grid.spacing,
            );
        }

        MouseInfo {
            button: button.unwrap_or(i.mouse_info.button),
            is_down: mousedown.unwrap_or(i.mouse_info.is_down),
            modifiers: command_mod,
            position: mposition,
            absolute_position: absolute_mposition,
            raw_position: raw_mposition,
            raw_absolute_position: raw_absolute_mposition,
        }
    }
}

impl Interface {
        // Generic events
    pub fn center_cursor(&mut self) {
        let mut center = self.sdl_window.drawable_size();
        center.0 /= 2;
        center.1 /= 2;
        self.mouse_info.absolute_position = (center.0 as f32, center.1 as f32);

        self.sdl_context
            .mouse()
            .warp_mouse_in_window(&self.sdl_window, center.0 as i32, center.1 as i32);
    }

}
