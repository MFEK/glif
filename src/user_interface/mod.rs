pub mod follow;
pub mod gui;
pub mod icons;
pub mod mouse_input;
pub mod sdl;
pub mod util;
pub mod egui_manager;

use std::rc::Rc;

use crate::render;
//use crate::user_interface::gui::build_imgui_ui;
use egui_sdl2_event::EguiSDL2State;
use egui_skia::EguiSkia;
use glifparser::MFEKPointData;
use glifrenderer::grid::Grid;
use glifrenderer::viewport::Viewport;
use sdl2::mouse::MouseState;
use sdl2::video::GLContext;
use skia_bindings::GrSurfaceOrigin;
use skia_safe::Color;
use skia_safe::ColorType;
use skia_safe::RCHandle;
use skia_bindings::GrDirectContext;
use skia_safe::Surface;
use skia_safe::gpu::BackendRenderTarget;
use skia_safe::gpu::gl::FramebufferInfo;

use crate::editor::Editor;
pub use crate::user_interface::mouse_input::MouseInfo;

use glifparser::glif::Layer;
use sdl2::{video::Window, Sdl};

use self::egui_manager::EguiManager;
use self::gui::build_ui;
use self::gui::window::WindowManager;

/* Window */
pub const PAPER_DRAW_GUIDELINES: bool = false;

pub struct Interface {
    prompts: Vec<InputPrompt>,
    sdl_context: Sdl,
    sdl_dpi: f32,
    pub sdl_window: Window,

    pub context: Option<(f32, f32)>,
    pub grid: Grid,
    pub mouse_info: MouseInfo,
    pub viewport: Viewport,

    // OpenGL and Skia
    pub gl_ctx: GLContext,
    pub gr_context: RCHandle<GrDirectContext>,
    pub fb_info: FramebufferInfo,
}

impl Interface {
    pub fn new(filename: &str) -> Self {
        let mut viewport = Viewport::default();
        let (sdl, window) = Self::initialize_sdl(filename, &mut viewport);

        let gl_ctx = window.gl_create_context().unwrap();
        gl::load_with(|name| sdl.video().unwrap().gl_get_proc_address(name) as *const _);

        let mut gr_context = skia_safe::gpu::DirectContext::new_gl(None, None).unwrap();

        let fb_info = {
            let mut fboid = 0;
            unsafe { gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid) };
    
            FramebufferInfo {
                fboid: fboid.try_into().unwrap(),
                format: skia_safe::gpu::gl::Format::RGBA8.into(),
            }
        };

        #[allow(unused_mut)]
        let mut iself = Interface {
            prompts: vec![],
            sdl_context: sdl,
            sdl_window: window,
            sdl_dpi: f32::NAN,

            context: None,
            grid: Grid::default(),
            mouse_info: MouseInfo::default(),
            viewport: Viewport::default(),

            gl_ctx,
            gr_context,
            fb_info,
        };


        iself.adjust_viewport_by_os_dpi();

        iself
    }

    pub fn create_surface(&mut self) -> skia_safe::Surface {
        create_surface(&self.sdl_window, &self.fb_info, &mut self.gr_context)
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

    pub fn extents(&mut self) -> (u32, u32) {
        let (window_width, window_height) = self.viewport.winsize;
        let (width, height) = (window_width.ceil() as u32, window_height.ceil() as u32);
        (width, height)
    }

    pub fn render(
        &mut self,
        v: &mut Editor,
        wm: &mut WindowManager,
        egui_manager: &mut EguiManager,
        sk_surface: &mut skia_safe::Surface,
    ) {
        build_ui(egui_manager, v, self, wm);
        let canvas = sk_surface.canvas();
        canvas.clear(Color::BLACK);
        render::render_frame(v, self, canvas);
        canvas.save();
        let scale = 1. / self.os_dpi();
        canvas.scale((scale, scale));
        egui_manager.egui.paint(canvas);
        canvas.restore();
        sk_surface.flush();
        self.sdl_window.gl_swap_window();
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


fn create_surface(
    window: &Window,
    fb_info: &FramebufferInfo,
    gr_context: &mut skia_safe::gpu::DirectContext,
) -> skia_safe::Surface {
    let (width, height) = window.drawable_size();

    let backend_render_target =
        BackendRenderTarget::new_gl((width as i32, height as i32), 0, 8, *fb_info);
    Surface::from_backend_render_target(
        gr_context,
        &backend_render_target,
        GrSurfaceOrigin::BottomLeft,
        ColorType::RGBA8888,
        None,
        None,
    )
    .unwrap()
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
