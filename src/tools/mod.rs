use pub_mod::pub_mod;
// Include all tools via procedural macro. Expands to `pub mod pen; pub mod select; ...`
pub_mod!("src/tools");

use self::prelude::*;
use self::{measure::Measure, pan::Pan, pen::Pen, select::Select, zoom::Zoom, vws::VWS, anchors::Anchors};
use dyn_clone::DynClone;
use imgui::Ui;

pub use self::zoom::{zoom_in_factor, zoom_out_factor};

use crate::editor::Editor;

pub trait Tool: DynClone{
    fn handle_event(&mut self, v: &mut Editor, event: EditorEvent);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToolEnum {
    Pan,
    Pen,
    Select,
    Zoom,
    Measure,
    VWS,
    Anchors,
}

pub fn tool_enum_to_tool(tool: ToolEnum) -> Box<dyn Tool> {
    match tool {
        ToolEnum::Pan => {Box::new(Pan::new())}
        ToolEnum::Pen => {Box::new(Pen::new())}
        ToolEnum::Select => {Box::new(Select::new())}
        ToolEnum::Zoom => {Box::new(Zoom::new())}
        ToolEnum::Measure => {Box::new(Measure::new())}
        ToolEnum::Anchors => {Box::new(Anchors::new())}
        ToolEnum::VWS => {Box::new(VWS::new())} //FIXME: enable vws
    }
}

pub enum MouseEventType {
    Pressed,
    DoubleClick,
    Released,
    Moved
}

pub enum EditorEvent<'a, 'b> {
    MouseEvent {
        event_type: MouseEventType,
        meta: MouseInfo
    },

    Draw {
        skia_canvas:  &'a mut Canvas
    },

    Ui {
        ui: &'b mut Ui<'a>
    }
}
