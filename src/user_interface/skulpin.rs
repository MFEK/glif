use sdl2::video::Window;
use skulpin::{rafx::api::RafxExtents2D, Renderer, RendererBuilder};

use super::{HEIGHT, Interface, WIDTH};

impl Interface {
    pub fn initialize_skulpin_renderer(window: &Window) -> Renderer {
        let (window_width, window_height) = window.vulkan_drawable_size();

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
            .build(window, extents);

        // TODO: Handle failure to initialize skulpin more gracefully.
        return renderer.unwrap();
    }
}
