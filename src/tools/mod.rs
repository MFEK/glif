// Include all tools via procedural macro. Expands to `pub mod pen; pub mod select; ...`
pub_mod!("src/tools");

use self::prelude::*;
use self::{
    anchors::Anchors, dash::Dash, grid::Grid, guidelines::Guidelines, image::Image,
    measure::Measure, pan::Pan, pap::PAP, pen::Pen, select::Select, shapes::Shapes, vws::VWS,
    zoom::Zoom,
};

use dyn_clone::DynClone;
use imgui::Ui;

use crate::editor::Editor;
use crate::user_interface::Interface;

pub trait Tool: DynClone + std::fmt::Debug + Send {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent);

    // We provide empty default implementations for these two because not every tool needs these hooks.
    fn draw(&mut self, _v: &Editor, _i: &Interface, _canvas: &mut Canvas) {}
    fn ui(&mut self, _v: &mut Editor, _i: &mut Interface, _ui: &mut Ui) {}
}

impl Default for Box<dyn Tool> {
    fn default() -> Self {
        Box::new(Pan::new())
    }
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
        PAP,
        Dash,
        Shapes,
        Image,
        Guidelines
    }
}

impl Default for ToolEnum {
    fn default() -> Self {
        ToolEnum::Pan
    }
}

pub fn tool_enum_to_tool(tool: ToolEnum) -> Box<dyn Tool> {
    match tool {
        ToolEnum::Pan => Box::new(Pan::new()),
        ToolEnum::Pen => Box::new(Pen::new()),
        ToolEnum::Select => Box::new(Select::new()),
        ToolEnum::Zoom => Box::new(Zoom::new()),
        ToolEnum::Anchors => Box::new(Anchors::new()),
        ToolEnum::Grid => Box::new(Grid::new()),
        ToolEnum::Measure => Box::new(Measure::new()),
        ToolEnum::Shapes => Box::new(Shapes::new()),
        ToolEnum::VWS => Box::new(VWS::new()),
        ToolEnum::Dash => Box::new(Dash::new()),
        ToolEnum::Image => Box::new(Image::new()),
        ToolEnum::PAP => Box::new(PAP::new()),
        ToolEnum::Guidelines => Box::new(Guidelines::new()),
    }
}
