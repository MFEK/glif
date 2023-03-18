use sdl2::video::GLContext;
use sdl2::{video::Window as SdlWindow};

use super::Interface;

pub struct PopoutWindow<'a> {
    pub gl_ctx: &'a mut GLContext,
    pub sdl_window: &'a mut SdlWindow,
}

pub trait Popout<'a> {
    fn popout(&mut self) -> PopoutWindow<'a>;
}

impl<'a> Popout<'a> for Interface {
    fn popout(&mut self) -> PopoutWindow<'a> {
        todo!()
    }
}
