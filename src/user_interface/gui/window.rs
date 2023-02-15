use egui::Context;

use crate::{editor::Editor, user_interface::Interface};
use crate::user_interface::gui::windows::inspection_window::InspectionWindow;

use super::windows::grid_window::GridWindow;
use super::windows::layer_list::LayerList;
use super::windows::tool_window::ToolWindow;
pub struct WindowManager {
    pub inspector: InspectionWindow,
    pub grid: GridWindow,
    pub tool: ToolWindow,
    pub layer_list: LayerList,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            inspector: InspectionWindow::new(),
            grid: GridWindow::new(),
            tool: ToolWindow::new(),
            layer_list: LayerList::new(),
        }
    }
}
pub trait GlifWindow {
    fn open(&self) -> bool;
    fn set_open(&mut self, open: bool);
    
    fn build(&mut self, ctx: &Context, v: &mut Editor, i: &mut Interface);
}