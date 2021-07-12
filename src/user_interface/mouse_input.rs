use sdl2::mouse::MouseButton;
use crate::renderer::points::calc::*;

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
            let mpos = (mposition.0, calc_y(mposition.1));

            let mut candidates = vec![];

            let standard_snap = (
                (mpos.0 / grid.spacing + grid.offset).round() * grid.spacing,
                calc_y((mpos.1 / grid.spacing + grid.offset).round() * grid.spacing),
            );

            let dist = f32::sqrt(f32::powi(standard_snap.0 - mpos.0, 2) + f32::powi(standard_snap.1 - mposition.1, 2));

            candidates.push((
                dist,
                standard_snap
            ));

            if let Some(slope) = &grid.slope {
                let slope_max = f32::min(*slope, 1.);
                let x = mpos.0 - mpos.1/slope;
                let s = (grid.spacing/slope_max).abs();
                let c = (x/s + 0.5).floor() * s;
                let c2 = c * -slope;

                let closest_italic = ((mpos.1 - c2)/slope, mpos.1);

                let horizontal_candidate = ((rcalc_y(standard_snap.1) - c2)/slope, standard_snap.1);
                let dist = f32::sqrt(f32::powi(horizontal_candidate.0 - mpos.0, 2) + f32::powi(horizontal_candidate.1 - mposition.1, 2));

                candidates.push((
                    dist,
                    horizontal_candidate
                ));

                let vertical_candidate = (standard_snap.0, rcalc_y(slope*standard_snap.0+c2));
                let dist = f32::sqrt(f32::powi(vertical_candidate.0 - mpos.0, 2) + f32::powi(vertical_candidate.1 - mposition.1, 2));

                candidates.push((
                    dist,
                    vertical_candidate
                ));
            }

            candidates.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            mposition = candidates[0].1;
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
