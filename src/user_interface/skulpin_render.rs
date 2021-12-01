use sdl2::video::Window;
use skulpin::{rafx::api::RafxExtents2D, CoordinateSystem, CoordinateSystemHelper, Renderer, RendererBuilder, skia_safe as sk};

use super::{Interface, HEIGHT, WIDTH};

impl Interface {
    pub const fn default_extents() -> RafxExtents2D {
        RafxExtents2D {
            width: WIDTH,
            height: HEIGHT,
        }
    }

    pub fn default_coordinate_system_helper() -> CoordinateSystemHelper {
        CoordinateSystemHelper::new(Self::default_extents(), 1.0)
    }

    pub fn initialize_skulpin_renderer(window: &Window) -> Renderer {
        let chs = Self::default_coordinate_system_helper();

        let renderer = RendererBuilder::new()
            .coordinate_system(CoordinateSystem::None)
            .build(window, Self::default_extents());

        renderer.expect("Failed to initialize Skulpin")
    }
}
