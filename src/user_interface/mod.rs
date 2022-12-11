pub mod follow;
pub mod gui;
pub mod icons;
pub mod mouse_input;
pub mod sdl;
pub mod skulpin_render;
pub mod util;

use std::rc::Rc;

use crate::render;
use crate::user_interface::gui::build_imgui_ui;
use glifparser::MFEKPointData;
use glifrenderer::grid::Grid;
use glifrenderer::viewport::Viewport;
use imgui::{self, Context};
use imgui_sdl2::ImguiSdl2;
use sdl2::mouse::MouseState;
use skulpin::rafx::api::RafxExtents2D;
use skulpin::Renderer;

use crate::editor::Editor;
pub use crate::user_interface::mouse_input::MouseInfo;

use glifparser::glif::Layer;
use sdl2::{video::Window, Sdl};

pub use self::gui::ImguiManager;
use self::gui::LAYERBOX_HEIGHT;
use self::gui::LAYERBOX_WIDTH;
use self::gui::TOOLBOX_OFFSET_X;
use self::gui::TOOLBOX_OFFSET_Y;

/* Window */
pub const PAPER_DRAW_GUIDELINES: bool = false;

pub struct Interface {
    prompts: Vec<InputPrompt>,
    sdl_context: Sdl,
    sdl_dpi: f32,
    pub sdl_window: Window,

    pub grid: Grid,
    pub mouse_info: MouseInfo,
    pub viewport: Viewport,
}

impl Interface {
    pub fn new(filename: &str) -> Self {
        let mut viewport = Viewport::default();
        let (sdl, window) = Self::initialize_sdl(filename, &mut viewport);

        #[allow(unused_mut)]
        let mut iself = Interface {
            prompts: vec![],
            sdl_context: sdl,
            sdl_window: window,
            sdl_dpi: f32::NAN,

            grid: Grid::default(),
            mouse_info: MouseInfo::default(),
            viewport: Viewport::default(),
        };

        iself.adjust_viewport_by_os_dpi();

        iself
    }

    pub fn adjust_viewport_by_os_dpi(&mut self) {
        let dpi = self.os_dpi();
        self.viewport.winsize.0 /= dpi;
        self.viewport.winsize.1 /= dpi;
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

    pub fn get_inspector_dialog_rect(&self) -> (f32, f32, f32, f32) {
        let (tx, ty, tw, th) = self.get_tools_dialog_rect();
        (tx, ty - (th * 0.35), tw, (th * 0.35) - (TOOLBOX_OFFSET_Y))
    }

    pub fn get_tools_dialog_rect(&self) -> (f32, f32, f32, f32) {
        let offset_y =
            (self.viewport.winsize.1 as f32 * self.sdl_dpi - (LAYERBOX_HEIGHT * 2.)) / 3.;
        (
            self.viewport.winsize.0 as f32 * self.sdl_dpi - (LAYERBOX_WIDTH) - (TOOLBOX_OFFSET_X),
            self.viewport.winsize.1 as f32 * self.sdl_dpi
                - (LAYERBOX_HEIGHT * 2.)
                - (TOOLBOX_OFFSET_Y * 2.)
                - offset_y,
            LAYERBOX_WIDTH,
            LAYERBOX_HEIGHT + offset_y,
        )
    }

    pub fn extents(&mut self) -> RafxExtents2D {
        let (window_width, window_height) = self.viewport.winsize;
        let (width, height) = (window_width.ceil() as u32, window_height.ceil() as u32);
        RafxExtents2D { width, height }
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
        // What we are doing with Viewport is far too complex for skulpin::CoordinateSystemHelper,
        // thus we've stubbed it out.
        let drew = skulpin.draw(self.extents(), 1.0, |canvas, _| {
            render::render_frame(v, self, canvas);
            canvas.save();
            let scale = 1. / self.os_dpi();
            canvas.scale((scale, scale));
            imgui_renderer.render_imgui(canvas, dd);
            canvas.restore();
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

    fn set_dpi_from_os(&mut self) -> f32 {
        let (w, h) = self.sdl_window.drawable_size();
        let hdpi = self.viewport.winsize.0 / w as f32;
        let vdpi = self.viewport.winsize.1 / h as f32;
        if hdpi != vdpi {
            log::warn!("Warning: DPI's not equal? {} != {}", hdpi, vdpi);
        }
        self.sdl_dpi = hdpi;
        hdpi
    }

    pub fn os_dpi(&mut self) -> f32 {
        if self.sdl_dpi.is_nan() {
            self.set_dpi_from_os()
        } else {
            self.sdl_dpi
        }
    }

    pub fn update_viewport(&mut self, offset: Option<(f32, f32)>, scale: Option<f32>) {
        let offset = match offset {
            None => self.viewport.offset,
            Some(offset) => (offset.0, offset.1),
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
    YesNo {
        question: String,
        afterword: String,
        func: Rc<dyn Fn(&mut Editor, &mut Interface, bool)>,
    },
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
