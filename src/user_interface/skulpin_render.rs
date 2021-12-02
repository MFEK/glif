use sdl2::video::Window;
use skulpin::{rafx::api::RafxExtents2D, CoordinateSystem, Renderer, RendererBuilder};

use super::{Interface, HEIGHT, WIDTH};

impl Interface {
    pub const fn default_extents() -> RafxExtents2D {
        RafxExtents2D {
            width: WIDTH,
            height: HEIGHT,
        }
    }

    pub fn initialize_skulpin_renderer(window: &Window) -> Renderer {
        let renderer = RendererBuilder::new()
            .coordinate_system(CoordinateSystem::None)
            .build(window, Self::default_extents());

        renderer.expect("Failed to initialize Skulpin")
    }
}
