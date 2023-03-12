use std::collections::HashSet;
use std::ffi::NulError;

use crate::user_interface::Interface;

use glifrenderer::viewport::Viewport;
use image;
use sdl2::keyboard::Keycode;
use sdl2::video::{GLContext, GLProfile};
use sdl2::EventPump;
use sdl2::{pixels::PixelFormatEnum, surface::Surface, video::Window, Sdl};
use skia_bindings::GrDirectContext;
use skia_safe::RCHandle;

impl Interface {
    // for macOS, we may mutate viewport.winsize. other OS don't (normally?) mutate viewport
    pub fn initialize_sdl(
        filename: &str,
        viewport: &mut Viewport,
    ) -> (Sdl, Window, RCHandle<GrDirectContext>, GLContext) {
        // SDL initialization
        let sdl_context = sdl2::init().expect("Failed to initialize sdl2");
        let video_subsystem = sdl_context
            .video()
            .expect("Failed to create sdl video subsystem");

        let mut window = video_subsystem
            .window(
                &format!("MFEKglif — {}", filename),
                viewport.winsize.0 as u32,
                viewport.winsize.1 as u32,
            )
            .opengl()
            .position_centered()
            .allow_highdpi()
            .resizable()
            .build()
            .expect("Failed to create SDL Window");

        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 3);
        debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
        debug_assert_eq!(gl_attr.context_version(), (3, 3));

        let gl_ctx = window.gl_create_context().unwrap();
        gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);
        let interface = skia_safe::gpu::gl::Interface::new_load_with(|name| {
            if name == "eglGetCurrentDisplay" {
                return std::ptr::null();
            }
            video_subsystem.gl_get_proc_address(name) as *const _
        })
        .expect("Could not create interface");

        let gr_context = skia_safe::gpu::DirectContext::new_gl(Some(interface), None).unwrap();
        video_subsystem.text_input().start();

        let logo = include_bytes!("../../resources/icon.png");

        let mut im = image::load_from_memory_with_format(logo, image::ImageFormat::Png)
            .unwrap()
            .into_rgba8();

        // SDL2's pixel formats are not byte-by-byte, but rather word-by-word, where the words are each
        // 32 bits long. So RGBA8888 means a 32-bit word where 8 bits are R, G, B and A. However,
        // SDL2's words are not big endian, they are little endian, so we need to reverse them.
        im.chunks_exact_mut(4).for_each(|pixel: &mut _| {
            let oldpixel: [u8; 4] = [pixel[0], pixel[1], pixel[2], pixel[3]];
            pixel[0] = oldpixel[3];
            pixel[1] = oldpixel[2];
            pixel[2] = oldpixel[1];
            pixel[3] = oldpixel[0];
        });

        let surface = Surface::from_data(&mut im, 512, 512, 512 * 4, PixelFormatEnum::RGBA8888)
            .expect("Failed to create SDL2 Surface");

        window.set_icon(surface);

        (sdl_context, window, gr_context, gl_ctx)
    }

    fn create_gl_context(&mut self) {}

    pub fn set_window_title(&mut self, title: &str) -> Result<(), NulError> {
        self.sdl_window.set_title(title)
    }

    pub fn get_event_pump(&self) -> EventPump {
        self.sdl_context
            .event_pump()
            .expect("Could not create sdl event pump")
    }

    pub fn get_pressed_keys(&self, event_pump: &EventPump) -> HashSet<Keycode> {
        event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect::<HashSet<Keycode>>()
    }
}
