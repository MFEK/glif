pub mod prelude;
use self::{measure::Measure, pan::Pan, pen::Pen, prelude::*, select::Select, zoom::Zoom};
use dyn_clone::DynClone;
use imgui::Ui;

pub mod console;

pub mod pan;
pub mod pen;
pub mod select;
//pub mod vws;
pub mod zoom;
pub mod measure;

pub use self::zoom::{zoom_in_factor, zoom_out_factor};
use crate::{command::CommandMod};
use sdl2::video::Window;
use sdl2::{mouse::MouseButton, Sdl};

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
    Released,
    Moved
}

pub enum EditorEvent<'a> {
    MouseEvent {
        event_type: MouseEventType,
        position: (f64, f64),
        meta: MouseMeta
    },

    Draw {
        skia_canvas:  &'a mut Canvas
    },

    Ui {
        ui: &'a mut Ui<'a>
    }
}

pub trait Tool: DynClone{
    fn handle_event(&mut self, v: &mut state::Editor, event: EditorEvent);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseMeta {
    pub modifiers: CommandMod,
    pub button: MouseButton,
}

// Generic events
pub fn center_cursor(v: &mut state::Editor, sdl_context: &Sdl, sdl_window: &Window) {
    let mut center = sdl_window.size();
    center.0 /= 2;
    center.1 /= 2;
    v.absolute_mousepos = (center.0 as f64, center.1 as f64);

    sdl_context
        .mouse()
        .warp_mouse_in_window(&sdl_window, center.0 as i32, center.1 as i32);
}

// this gets called by tools so it accepts &mut State
pub fn update_viewport(
    v: &mut state::Editor,
    offset: Option<(f32, f32)>,
    scale: Option<f32>,
) {
    let offset = match offset {
        None => v.offset,
        Some(offset) => offset,
    };
    let scale = match scale {
        None => v.factor,
        Some(scale) => scale,
    };
    v.factor = scale;
    v.offset = offset;
}

// this only gets called prior to events in the main loop so it recieves an unborrowed state
pub fn update_mousepos(
    v: &mut state::Editor,
    position: (f64, f64),
    mousedown: Option<bool>
) {
    let factor = 1. / v.factor as f64;
    let uoffset = v.offset;
    let offset = (uoffset.0 as f64, uoffset.1 as f64);

    let absolute_mposition = ((position.0).floor(), (position.1).floor());
    let mposition = (
        ((position.0).floor() - offset.0) * factor,
        ((position.1).floor() - offset.1) * factor,
    );

    v.absolute_mousepos = absolute_mposition;
    v.mousepos = mposition;
    
    if let Some(mousedown) = mousedown {
        v.mousedown = mousedown;
    }
}

pub fn mode_switched(v: &mut state::Editor, from: ToolEnum, to: ToolEnum) {
    v.contour_idx = None;
    v.point_idx = None;
}

#[macro_export]
///! Given a field on the State struct, and an enumerator that implements IntoEnumIterator, cycle
///! through its variants and update state. An optional condition is provided. $state is expected to
///! be an inner thread::LocalKey<State>.
macro_rules! trigger_toggle_on {
    ($state:ident, $state_var:ident, $enum:ident, $cond:expr) => {
        let $state_var = $state.$state_var;
        if $cond {
            let mut e = $enum::into_enum_iter()
                .cycle()
                .skip(1 + $state_var as usize);
            let n = e.next().unwrap();
            $state.$state_var = n;
        }
    };
    ($state:ident, $state_var:ident, $enum:ident) => {
        trigger_toggle_on!($state, $state_var, $enum, true);
    };
}
