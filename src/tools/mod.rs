//use pub_mod::pub_mod;
// Include all tools via procedural macro. Expands to `pub mod pen; pub mod select; ...`
//pub_mod!("src/tools");

// TODO: reenable pub_mod! before this hits the main branch.
pub mod measure;
pub mod pan;
pub mod pen;
pub mod select;
pub mod zoom;
pub mod anchors;
pub mod pap;
pub mod grid;
pub mod image;
pub mod guidelines;
pub mod prelude;
pub mod console;
pub mod shapes;
pub mod vws;

use self::prelude::*;
use self::{
    measure::Measure,
    pan::Pan,
    pen::Pen,
    select::Select,
    zoom::Zoom,
    vws::VWS,
    anchors::Anchors,
    shapes::Shapes,
    pap::PAP,
    grid::GridTool,
    image::Image,
    guidelines::Guidelines,
};

use dyn_clone::DynClone;
use imgui::Ui;
use crate::user_interface::Interface;
//pub use self::zoom::{zoom_in_factor, zoom_out_factor};

use crate::command::{Command, CommandMod};
use crate::editor::Editor;

pub trait Tool: DynClone {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent);

    // We provide empty default implementations for these two because not every tool needs these hooks.
    fn draw(&self, _v: &Editor, _i: &Interface, _canvas: &mut Canvas) {}
    fn ui(&mut self, _v: &mut Editor, _i: &mut Interface, _ui: &mut Ui) {}
}

use enum_unitary::enum_unitary;
enum_unitary! {
    #[derive(Debug, Copy, PartialEq)]
    pub enum ToolEnum {
        Pan,
        Pen,
        Select,
        Grid,
        Anchors,
        Zoom,
        Measure,
        VWS,
        Shapes,
        Image,
        PAP,
        Guidelines
    }
}

pub fn tool_enum_to_tool(tool: ToolEnum) -> Box<dyn Tool> {
    match tool {
        ToolEnum::Pan => {Box::new(Pan::new())}
        ToolEnum::Pen => {Box::new(Pen::new())}
        ToolEnum::Select => {Box::new(Select::new())}
        ToolEnum::Zoom => {Box::new(Zoom::new())}
        ToolEnum::Anchors => {Box::new(Anchors::new())}
        ToolEnum::Grid => {Box::new(GridTool::new())}
        ToolEnum::Measure => {Box::new(Measure::new())}
        ToolEnum::Shapes => {Box::new(Shapes::new())} //FIXME: enable vws
        ToolEnum::VWS => {Box::new(VWS::new())} //FIXME: enable vws
        ToolEnum::Image => {Box::new(Image::new())}
        ToolEnum::PAP => {Box::new(PAP::new())}
        ToolEnum::Guidelines => {Box::new(Guidelines::new())}
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
        mouse_info: MouseInfo
    },

    ToolCommand {
        command: Command,
        command_mod: CommandMod,
        stop_after: &'a mut bool,
    },
}
