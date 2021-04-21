pub mod prelude;
use self::prelude::*;
use self::{measure::Measure, pan::Pan, pen::Pen, select::Select, zoom::Zoom};
use dyn_clone::DynClone;
use imgui::Ui;

pub use self::zoom::{zoom_in_factor, zoom_out_factor};
use crate::command::CommandMod;

use crate::editor::Editor;

use sdl2::video::Window;
use sdl2::{mouse::MouseButton, Sdl};

pub mod console;

pub mod pan;
pub mod pen;
pub mod select;
//pub mod vws;
pub mod zoom;
pub mod measure;



#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToolEnum {
    Pan,
    Pen,
    Select,
    Zoom,
    Measure,
    VWS,
}

pub fn tool_enum_to_tool(tool: ToolEnum) -> Box<dyn Tool> {
    match tool {
        ToolEnum::Pan => {Box::new(Pan::new())}
        ToolEnum::Pen => {Box::new(Pen::new())}
        ToolEnum::Select => {Box::new(Select::new())}
        ToolEnum::Zoom => {Box::new(Zoom::new())}
        ToolEnum::Measure => {Box::new(Measure::new())}
        ToolEnum::VWS => {Box::new(Pan::new())} //FIXME: enable vws
    }
}

pub enum MouseEventType {
    Pressed,
    DoubleClick,
    Released,
    Moved
}

pub enum EditorEvent<'a> {
    MouseEvent {
        event_type: MouseEventType,
        meta: MouseInfo
    },

    Draw {
        skia_canvas:  &'a mut Canvas
    },

    Ui {
        ui: &'a mut Ui<'a>
    }
}

pub trait Tool: DynClone{
    fn handle_event(&mut self, v: &mut Editor, event: EditorEvent);
}

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
            modifiers: CommandMod{ shift: false, ctrl: false }
        }
    }
}

impl MouseInfo {
    pub fn new(
        v: &Editor,
        last_meta: MouseInfo,
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
            button: button.unwrap_or(last_meta.button),
            is_down: mousedown.unwrap_or(last_meta.is_down),
            modifiers: command_mod,
            position: mposition,
            absolute_position: absolute_mposition
        }
    }
}

// Generic events
pub fn _center_cursor(v: &mut Editor, sdl_context: &Sdl, sdl_window: &Window) {
    let mut center = sdl_window.size();
    center.0 /= 2;
    center.1 /= 2;
    v.mouse_info.absolute_position = (center.0 as f32, center.1 as f32);

    sdl_context
        .mouse()
        .warp_mouse_in_window(&sdl_window, center.0 as i32, center.1 as i32);
}

// this gets called by tools so it accepts &mut State
pub fn update_viewport(
    v: &mut Editor,
    offset: Option<(f32, f32)>,
    scale: Option<f32>,
) {
    let offset = match offset {
        None => v.viewport.offset,
        Some(offset) => offset,
    };
    let scale = match scale {
        None => v.viewport.factor,
        Some(scale) => scale,
    };

    v.viewport.factor = scale;
    v.viewport.offset = offset;
}

#[macro_export]
///! Given a field on the State struct, and an enumerator that implements IntoEnumIterator, cycle
///! through its variants and update state. An optional condition is provided. $state is expected to
///! be an inner thread::LocalKey<State>.
macro_rules! trigger_toggle_on {
    ($state:ident, $state_var:ident, $enum:ident, $cond:expr) => {
        let $state_var = $state.viewport.$state_var;
        if $cond {
            let mut e = $enum::into_enum_iter()
                .cycle()
                .skip(1 + $state_var as usize);
            let n = e.next().unwrap();
            $state.viewport.$state_var = n;
        }
    };
    ($state:ident, $state_var:ident, $enum:ident) => {
        trigger_toggle_on!($state, $state_var, $enum, true);
    };
}
