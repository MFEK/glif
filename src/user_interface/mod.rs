pub mod follow;
pub mod grid;
pub mod gui;
pub mod icons;
pub mod mouse_input;
pub mod sdl;
pub mod skulpin_render;
pub mod util;

use std::rc::Rc;

use crate::render;
use crate::user_interface::gui::build_imgui_ui;
use glifrenderer::viewport::Viewport;
use imgui::{self, Context};
use imgui_sdl2::ImguiSdl2;
use sdl2::mouse::MouseState;
use skulpin::rafx::api::RafxExtents2D;
use skulpin::Renderer;

use crate::editor::Editor;
pub use crate::user_interface::mouse_input::MouseInfo;
use glifparser::glif::{Layer, MFEKPointData};
use sdl2::{video::Window, Sdl};

use self::grid::Grid;
pub use self::gui::ImguiManager;
use self::gui::LAYERBOX_HEIGHT;
use self::gui::LAYERBOX_WIDTH;
use self::gui::TOOLBOX_OFFSET_X;
use self::gui::TOOLBOX_OFFSET_Y;

/* Window */
pub const HEIGHT: u32 = 800;
pub const WIDTH: u32 = HEIGHT;
pub const PAPER_DRAW_GUIDELINES: bool = false;

pub struct Interface {
    prompts: Vec<InputPrompt>,
    sdl_context: Sdl,
    pub sdl_window: Window,

    pub grid: Option<Grid>,
    pub mouse_info: MouseInfo,
    pub viewport: Viewport,
}

impl Interface {
    pub fn new(filename: &str) -> Self {
        let (sdl, window) = Self::initialize_sdl(filename);

        Interface {
            prompts: vec![],
            sdl_context: sdl,
            sdl_window: window,

            grid: None,
            mouse_info: MouseInfo::default(),
            viewport: Viewport::default().with_winsize((WIDTH as f32, HEIGHT as f32)),
        }
    }

    pub fn active_prompts(&self) -> bool {
        !self.prompts.is_empty()
    }

    pub fn peek_prompt(&self) -> &InputPrompt {
        &self.prompts.first().unwrap()
    }

    pub fn pop_prompt(&mut self) -> Option<InputPrompt> {
        self.prompts.pop()
    }

    pub fn get_tools_dialog_rect(&self) -> (f32, f32, f32, f32) {
        let offset_y = (self.viewport.winsize.1 as f32 - (LAYERBOX_HEIGHT * 2.)) / 3.;
        (
            self.viewport.winsize.0 as f32 - (LAYERBOX_WIDTH) - (TOOLBOX_OFFSET_X),
            self.viewport.winsize.1 as f32
                - (LAYERBOX_HEIGHT * 2.)
                - (TOOLBOX_OFFSET_Y * 2.)
                - offset_y,
            LAYERBOX_WIDTH,
            LAYERBOX_HEIGHT + offset_y,
        )
    }

    pub fn render(
        &mut self,
        v: &mut Editor,
        imgui: &mut Context,
        imsdl2: &mut ImguiSdl2,
        imgui_renderer: &mut imgui_skia_renderer::Renderer,
        skulpin: &mut Renderer,
        mouse_state: &MouseState,
    ) {
        // build and render imgui
        let dd = build_imgui_ui(imgui, imsdl2, v, self, mouse_state);

        // draw glyph preview and imgui with skia
        let (window_width, window_height) = self.sdl_window.vulkan_drawable_size();
        let extents = RafxExtents2D {
            width: window_width,
            height: window_height,
        };

        let drew = skulpin.draw(extents, 1.0, |canvas, _coordinate_system_helper| {
            render::render_frame(v, self, canvas);
            imgui_renderer.render_imgui(canvas, dd);
        });

        if drew.is_err() {
            log::warn!("Failed to draw frame. This can happen when resizing due to VkError(ERROR_DEVICE_LOST); if happens otherwise, file an issue.");
        }
    }

    pub fn push_prompt(&mut self, prompt: InputPrompt) {
        self.prompts.push(prompt);
    }

    pub fn nudge_viewport(&mut self, offset: (f32, f32)) {
        let now_offset = self.viewport.offset;
        self.viewport.offset = (now_offset.0 + offset.0, now_offset.1 + offset.1);
    }

    pub fn update_viewport(&mut self, offset: Option<(f32, f32)>, scale: Option<f32>) {
        let offset = match offset {
            None => self.viewport.offset,
            Some(offset) => (
                offset.0,
                offset.1,
            ),
        };
        let scale = match scale {
            None => self.viewport.factor,
            Some(scale) => scale,
        };

        self.viewport.factor = scale;
        self.viewport.offset = offset;
    }
}

#[derive(Clone)]
pub enum InputPrompt {
    Text {
        label: String,
        default: String,
        func: Rc<dyn Fn(&mut Editor, String)>,
    },
    Color {
        label: String,
        default: [f32; 4],
        func: Rc<dyn Fn(&mut Editor, Option<[f32; 4]>)>,
    },
    Layer {
        label: String,
        func: Rc<dyn Fn(&mut Editor, Layer<MFEKPointData>)>,
    },
}
