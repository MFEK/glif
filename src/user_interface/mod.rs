use std::{rc::Rc};

use imgui::{self, Context};
use imgui_sdl2::ImguiSdl2;
use sdl2::mouse::MouseState;
use ::skulpin::Renderer;
use ::skulpin::rafx::api::RafxExtents2D;


use crate::renderer;
use crate::editor::Editor;
use crate::renderer::constants::HEIGHT;
use crate::renderer::constants::WIDTH;
pub use crate::user_interface::mouse_input::MouseInfo;
use glifparser::glif::{Layer, MFEKPointData};
use sdl2::{Sdl, video::Window};

pub use self::gui::ImguiManager;
use self::gui::LAYERBOX_HEIGHT;
use self::gui::LAYERBOX_WIDTH;
use self::gui::TOOLBOX_OFFSET_X;
use self::gui::TOOLBOX_OFFSET_Y;
use self::viewport::Viewport;

pub mod icons;
pub mod sdl;
pub mod gui;
pub mod viewport;
pub mod skulpin;
pub mod mouse_input;

pub struct Interface {
    prompts: Vec<InputPrompt>,
    sdl_context: Sdl,
    pub sdl_window: Window,

    pub mouse_info: MouseInfo,
    pub viewport: Viewport,
}

impl Interface {
    pub fn new(filename: &str) -> Self {
        let (sdl, window) = Self::initialize_sdl(filename);

        let mut ret = Interface {
            prompts: vec![],
            sdl_context: sdl,
            sdl_window: window,

            mouse_info: MouseInfo::default(),
            viewport: Viewport::default(),
        };

        ret.viewport.winsize = (WIDTH as u32, HEIGHT as u32);

        return ret;
    }

    pub fn active_prompts(&self) -> bool {
        return !self.prompts.is_empty();
    }

    pub fn peek_prompt(&self) -> &InputPrompt {
        return &self.prompts.first().unwrap();
    }
    
    pub fn pop_prompt(&mut self) {
        self.prompts.pop();
    }

    pub fn get_tools_dialog_rect(&self) -> (f32, f32, f32, f32) {
        (
            self.viewport.winsize.0 as f32 - (LAYERBOX_WIDTH) - (TOOLBOX_OFFSET_X),
            self.viewport.winsize.1 as f32 - (LAYERBOX_HEIGHT * 2.) - (TOOLBOX_OFFSET_Y * 2.),
            LAYERBOX_WIDTH,
            LAYERBOX_HEIGHT,
        )
    }

    pub fn render(&mut self, v: &mut Editor, imgui: &mut Context, imsdl2: &mut ImguiSdl2, imgui_renderer: &mut imgui_skia_renderer::Renderer, skulpin: &mut Renderer, mouse_state: &MouseState) {
        // build and render imgui
        let dd = ImguiManager::build_imgui_ui(imgui, imsdl2, v, self, &mouse_state);

        // draw glyph preview and imgui with skia
        let (window_width, window_height) = self.sdl_window.vulkan_drawable_size();
        let extents = RafxExtents2D {
            width: window_width,
            height: window_height,
        };

        let drew = skulpin.draw(extents, 1.0, |canvas, _coordinate_system_helper| {
            renderer::render_frame(v, self, canvas);
            imgui_renderer.render_imgui(canvas, dd);
        });

        if drew.is_err() {
            log::warn!("Failed to draw frame. This can happen when resizing due to VkError(ERROR_DEVICE_LOST); if happens otherwise, file an issue.");
        }
    }

    pub fn push_prompt(&mut self, prompt: InputPrompt) {
        self.prompts.push(prompt);
    }
}

#[derive(Clone)]
pub enum InputPrompt {
    Text {
        label: String,
        default: String,
        func: Rc<dyn Fn(&mut Editor, String)>
    },
    Color {
        label: String,
        default: [f32; 4],
        func: Rc<dyn Fn(&mut Editor, Option<[f32; 4]>)>
    },
    Layer {
        label: String,
        func: Rc<dyn Fn(&mut Editor, Layer<MFEKPointData>)>
    }
}