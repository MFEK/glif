use sdl2::{video::Window, Sdl, surface::Surface, pixels::PixelFormatEnum};
use skulpin::{rafx::api::RafxError, rafx::api::RafxExtents2D, LogicalSize, RendererBuilder};
use image;

use crate::editor::Editor;
use crate::renderer::constants::*;

pub fn initialize_sdl(v: &mut Editor, filename: &str) -> (Sdl, Window) {
    // SDL initialization
    let sdl_context = sdl2::init().expect("Failed to initialize sdl2");
    let video_subsystem = sdl_context
        .video()
        .expect("Failed to create sdl video subsystem");

    video_subsystem.text_input().start();

    let logical_size = LogicalSize {
        width: WIDTH,
        height: HEIGHT,
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

    let logo = include_bytes!("../resources/icon.png");

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

    let surface = Surface::from_data(
        &mut im,
        512,
        512,
        512 * 4,
        PixelFormatEnum::RGBA8888,
    )
    .expect("Failed to create SDL2 Surface");

    window.set_icon(surface);

    v.viewport.winsize = (WIDTH as u32, HEIGHT as u32);

    (sdl_context, window)
}

pub fn initialize_skulpin_renderer(sdl_window: &Window) -> Result<skulpin::Renderer, RafxError> {
    let (window_width, window_height) = sdl_window.vulkan_drawable_size();

    let extents = RafxExtents2D {
        width: window_width,
        height: window_height,
    };

    let scale_to_fit = skulpin::skia_safe::matrix::ScaleToFit::Start;
    let visible_range = skulpin::skia_safe::Rect {
        left: 0.0,
        right: WIDTH as f32,
        top: 0.0,
        bottom: HEIGHT as f32,
    };

    let renderer = RendererBuilder::new()
        .coordinate_system(skulpin::CoordinateSystem::VisibleRange(
            visible_range,
            scale_to_fit,
        ))
        .build(sdl_window, extents);

    return renderer;
}
