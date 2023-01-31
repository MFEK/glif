pub mod layer_list;
pub mod prompts;
pub mod menu_bar;
pub mod tool_bar;
pub mod canvas_context;
pub mod window;
pub mod windows;

#[macro_use]
pub(crate) mod msgbox;

use std::cell::RefCell;
use crate::editor::Editor;
use super::{Interface, egui_manager::EguiManager};
use self::{window::{GlifWindow, WindowManager}};
pub(crate) use self::msgbox::gui_error as error;


pub fn build_ui(
    egui_manager: &mut EguiManager,
    v: &mut Editor,
    i: &mut Interface,
    wm: &mut WindowManager,
) {
    let egui = &mut egui_manager.egui;
    let egsdl2 = &mut egui_manager.egui_sdl2;
    egui.run(egsdl2.take_egui_input(&i.sdl_window), |ctx| {
        tool_bar::tool_bar(ctx, v, i);
        menu_bar::menu_bar(ctx, v, i, wm);
        layer_list::layer_list(ctx, v, i);

        // windows
        wm.inspector.build(ctx, v, i);

        if i.context.is_some() {
            canvas_context::canvas_context(ctx, v, i);
        }
    });
}

// Imgui globals
thread_local! { pub static PROMPT_STR: RefCell<String> = RefCell::new(String::new()); }
thread_local! { pub static PROMPT_CLR: RefCell<[f32; 4]> = RefCell::new([0., 0., 0., 1.]); }
