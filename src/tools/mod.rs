// Include all tools via procedural macro. Expands to `pub mod pen; pub mod select; ...`
pub_mod!("src/tools");

use self::prelude::*;
use self::{
    anchors::Anchors, grid::GridTool, guidelines::Guidelines, image::Image, measure::Measure,
    pan::Pan, pap::PAP, pen::Pen, select::Select, shapes::Shapes, vws::VWS, zoom::Zoom,
};

use dyn_clone::DynClone;
use imgui::Ui;

use crate::editor::Editor;
use crate::user_interface::Interface;

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
        ToolEnum::Pan => Box::new(Pan::new()),
        ToolEnum::Pen => Box::new(Pen::new()),
        ToolEnum::Select => Box::new(Select::new()),
        ToolEnum::Zoom => Box::new(Zoom::new()),
        ToolEnum::Anchors => Box::new(Anchors::new()),
        ToolEnum::Grid => Box::new(GridTool::new()),
        ToolEnum::Measure => Box::new(Measure::new()),
        ToolEnum::Shapes => Box::new(Shapes::new()), //FIXME: enable vws
        ToolEnum::VWS => Box::new(VWS::new()),       //FIXME: enable vws
        ToolEnum::Image => Box::new(Image::new()),
        ToolEnum::PAP => Box::new(PAP::new()),
        ToolEnum::Guidelines => Box::new(Guidelines::new()),
    }
}
