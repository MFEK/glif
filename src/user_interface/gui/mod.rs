pub mod icons;
pub use self::icons::build_and_add_button as build_and_add_icon_button;
pub use self::icons::build_button as build_icon_button;
pub use self::icons::IntoButtonResponse;
pub mod menu_bar;
#[macro_use]
pub(crate) mod msgbox;
pub(crate) use self::msgbox::gui_error as error;
pub mod prompts;
pub mod textedit_buffer;
pub mod tool_bar;
pub mod window;
pub mod windows;

use self::window::{GlifWindow, WindowManager};
use super::{egui_manager::EguiManager, Interface};
use crate::editor::Editor;
use std::cell::RefCell;

pub fn build_ui(
    egui_manager: &mut EguiManager,
    v: &mut Editor,
    i: &mut Interface,
    wm: &mut WindowManager,
) {
    // initialization (so far just sets fonts to system fonts not included Ubuntu)
    egui_manager.init();
    let egui = &mut egui_manager.egui;
    let egsdl2 = &mut egui_manager.egui_sdl2;
    egui.run(egsdl2.take_egui_input(&i.sdl_window), |ctx| {
        tool_bar::tool_bar(ctx, v, i);
        menu_bar::menu_bar(ctx, v, i, wm);

        // windows
        wm.layer_list.build(ctx, v, i);
        wm.inspector.build(ctx, v, i);
        wm.grid.build(ctx, v, i);
        wm.tool.build(ctx, v, i);

        if i.active_prompts() {
            prompts::build_and_check_prompts(v, i, ctx);
        }

        v.dispatch_ui(i, ctx);
    });
}

// Imgui globals
thread_local! { pub static PROMPT_STR: RefCell<String> = RefCell::new(String::new()); }
