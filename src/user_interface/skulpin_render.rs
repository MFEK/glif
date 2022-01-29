use skulpin::{CoordinateSystem, Renderer, RendererBuilder};

use super::Interface;

impl Interface {
    pub fn initialize_skulpin_renderer(&mut self) -> Renderer {
        let extents = self.extents();
        let renderer = RendererBuilder::new()
            .coordinate_system(CoordinateSystem::None)
            .build(&self.sdl_window, extents);

        renderer.expect("Failed to initialize Skulpin")
    }
}
