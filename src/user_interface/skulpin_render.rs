use sdl2::video::Window;
use skulpin::{CoordinateSystem, Renderer, RendererBuilder};

use super::Interface;

impl Interface {
    pub fn initialize_skulpin_renderer(&self, window: &Window) -> Renderer {
        let renderer = RendererBuilder::new()
            .coordinate_system(CoordinateSystem::None)
            .build(window, self.extents());

        renderer.expect("Failed to initialize Skulpin")
    }
}
