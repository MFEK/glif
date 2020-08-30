use glium::texture::SrgbTexture2d;
use super::reclutch_skia::{SkiaGraphicsDisplay, SkiaOpenGlTexture};
use crate::state::state;

use crate::renderer::constants::*;
use crate::renderer::render_frame;

use glium::GlObject;

pub fn make_skia_display(out_texture: &SrgbTexture2d, window_size: (u32, u32)) -> SkiaGraphicsDisplay {
    SkiaGraphicsDisplay::new_gl_texture(&SkiaOpenGlTexture {
        size: (window_size.0 as _, window_size.1 as _),
        texture_id: out_texture.get_id(),
        mip_mapped: false,
    })
    .expect("Failed to make Skia display")
}

// Redraw Skia. We could always do this since I'm requiring GPU, but some day I might change my
// mind on that, who knows. Plus, I'm being mindful of weaker GPU's than the one in my laptop,
// which was a quite expensive laptop in 2018. This has several practiacl effects: for one thing,
// our IMGui must not ever be transparent or semi-transparent, as it will cause flickering as IMGui
// will draw on top of itself as mouse moves around. Giving up transparency of IMGui for this speed
// boost is worth it to me; programming is all about tradeoffs.
pub fn redraw_skia(display: &mut SkiaGraphicsDisplay, should_redraw_skia: &mut bool) {
    let mut surface = &mut display.surface;
    let canvas = surface.canvas();
    let count = canvas.save();
    let center = (HEIGHT as f32 / 4., WIDTH as f32 / 4.);
    state.with(|v|render_frame(0, 12, 60, canvas));
    //canvas.restore_to_count(count);
    display.surface.flush_and_submit();
    *should_redraw_skia = false;
}
