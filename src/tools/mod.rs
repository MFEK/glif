// Include all tools via procedural macro. Expands to `pub mod pen; pub mod select; ...`
pub_mod!("src/tools");

use self::cut::Cut;
use self::prelude::*;
use self::{
    anchors::Anchors, dash::Dash, guidelines::Guidelines, image::Image, measure::Measure, pan::Pan,
    pap::PAP, pen::Pen, select::Select, shapes::Shapes, vws::VWS, zoom::Zoom,
};

use dyn_clone::DynClone;
use egui::Ui;

use crate::editor::Editor;
use crate::user_interface::Interface;

use std::fmt::{Display, Formatter};

pub trait Tool: DynClone + std::fmt::Debug {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent);

    // We provide empty default implementations for these two because not every tool needs these hooks.
    fn draw(&mut self, _v: &Editor, _i: &Interface, _canvas: &mut Canvas) {}

    // UI hooks. Dialog hooks into the tools dialog.
    fn dialog(&mut self, _v: &mut Editor, _i: &mut Interface, _ui: &mut Ui) -> bool {
        false
    }

    // TODO: Provide hooks for free floating UI (some tools might want this like pen for mode select)
    // and adding things to the right-click context menu.
    fn ui(&mut self, _v: &mut Editor, _i: &mut Interface, _ctx: &egui::Context) {}
}

impl Default for Box<dyn Tool> {
    fn default() -> Self {
        Box::new(Pan::new())
    }
}

use strum_macros::{AsRefStr, EnumString};
#[derive(Debug, Copy, Clone, AsRefStr, EnumString, PartialEq)]
pub enum ToolEnum {
    Pan,
    Pen,
    Cut,
    Select,
    Anchors,
    Zoom,
    Measure,
    VWS,
    PAP,
    Dash,
    Shapes,
    Image,
    Guidelines,
}

impl Display for ToolEnum {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        f.write_str(self.as_ref())
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
        ToolEnum::Measure => Box::new(Measure::new()),
        ToolEnum::Shapes => Box::new(Shapes::new()),
        ToolEnum::VWS => Box::new(VWS::new()),
        ToolEnum::Dash => Box::new(Dash::new()),
        ToolEnum::Image => Box::new(Image::new()),
        ToolEnum::PAP => Box::new(PAP::new()),
        ToolEnum::Guidelines => Box::new(Guidelines::new()),
        ToolEnum::Cut => Box::new(Cut::new()),
    }
}
