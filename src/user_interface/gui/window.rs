use egui::Context;

use crate::{editor::Editor, user_interface::Interface};
use crate::user_interface::gui::windows::inspection_window::InspectionWindow;
pub struct WindowManager {
    pub inspector: InspectionWindow,

}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            inspector: InspectionWindow::new()
        }
    }
}
pub trait GlifWindow {
    fn open(&self) -> bool;
    fn set_open(&mut self, open: bool);
    
    fn build(&mut self, ctx: &Context, v: &mut Editor, i: &mut Interface);
}