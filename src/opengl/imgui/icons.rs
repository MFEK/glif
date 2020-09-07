// SVG icons

use STATE;

use glium;
use nsvg;

mod data;

use self::data::{PAN_ICON_IMAGE, PEN_ICON_IMAGE, SELECT_ICON_IMAGE, ZOOM_ICON_IMAGE};
use glium::{texture, uniforms};
use glium::{Display, GlObject};
use imgui_glium_renderer::Renderer as ImguiRenderer;

use std::collections::HashMap;
use std::rc::Rc;

pub type Icons = HashMap<&'static str, (imgui::TextureId, Rc<texture::Texture2d>)>;

trait FromDisplay {
    fn from_display(display: &glium::Display, renderer: &mut ImguiRenderer) -> Self;
}

impl FromDisplay for Icons {
    #[rustfmt::skip]
    fn from_display(display: &glium::Display, renderer: &mut ImguiRenderer) -> Self {
        let mut ret = HashMap::new();
        ret.insert("pan", load_icon(display, renderer, PAN_ICON_IMAGE.clone()));
        ret.insert("pen", load_icon(display, renderer, PEN_ICON_IMAGE.clone()));
        ret.insert("select", load_icon(display, renderer, SELECT_ICON_IMAGE.clone()));
        ret.insert("zoom", load_icon(display, renderer, ZOOM_ICON_IMAGE.clone()));
        debug!("Loaded icons: {:?}", &ret);
        assert!(ret["pan"].1.get_id() > 0);
        assert!(ret["pen"].1.get_id() > 0);
        assert!(ret["select"].1.get_id() > 0);
        assert!(ret["zoom"].1.get_id() > 0);
        ret
    }
}

pub fn set_icons(renderer: &mut ImguiRenderer, gl_display: &Display) {
    STATE.with(|v| {
        let icons = Icons::from_display(gl_display, renderer);
        v.borrow_mut().icons = Some(icons);
    });
}

pub fn load_icon(
    display: &glium::Display,
    renderer: &mut ImguiRenderer,
    icon: data::SvgImageData,
) -> (imgui::TextureId, Rc<texture::Texture2d>) {
    let (width, height, image) = icon;
    let rawimage = texture::RawImage2d::from_raw_rgba(image, (width, height));
    let texture = Rc::new(
        texture::Texture2d::new(display, rawimage)
            .expect(&format!("Failed to upload texture for icon to GPU",)),
    );
    let id = renderer.textures().insert(texture.clone());
    (id, texture)
}
