use std::collections::HashSet;
use std::ffi::NulError;

use crate::user_interface::Interface;

use glifrenderer::viewport::Viewport;
use image;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use sdl2::{pixels::PixelFormatEnum, surface::Surface, video::Window, Sdl};
use skulpin::LogicalSize;

impl Interface {
    // for macOS, we may mutate viewport.winsize. other OS don't (normally?) mutate viewport
    pub fn initialize_sdl(filename: &str, viewport: &mut Viewport) -> (Sdl, Window) {
        // SDL initialization
        let sdl_context = sdl2::init().expect("Failed to initialize sdl2");
        let video_subsystem = sdl_context
            .video()
            .expect("Failed to create sdl video subsystem");

        video_subsystem.text_input().start();

        let logical_size = LogicalSize {
            width: viewport.winsize.0 as u32,
            height: viewport.winsize.1 as u32,
        };

        let mut window = video_subsystem
            .window(
                &format!("MFEKglif — {}", filename),
                logical_size.width,
                logical_size.height,
            )
            .position_centered()
            .allow_highdpi()
            .vulkan()
            .resizable()
            .build()
            .expect("Failed to create SDL Window");

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

        (sdl_context, window)
    }

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
