use sdl2::video::GLContext;
use sdl2::{video::Window as SdlWindow};

pub struct Popout<'a> {
    pub gl_ctx: &'a mut GLContext,
    pub sdl_window: &'a mut SdlWindow,
}
