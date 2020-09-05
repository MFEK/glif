// SVG icons

use glium;
use nsvg;

mod data;

use self::data::{PAN_ICON_IMAGE, PEN_ICON_IMAGE, SELECT_ICON_IMAGE, ZOOM_ICON_IMAGE};
use glium::GlObject;
use glium::{texture, uniforms};
use imgui_glium_renderer::Renderer as ImguiRenderer;

use std::rc::Rc;

#[derive(Debug)]
pub struct Icons {
    pub pan: (imgui::TextureId, Rc<texture::Texture2d>),
    pub pen: (imgui::TextureId, Rc<texture::Texture2d>),
    pub select: (imgui::TextureId, Rc<texture::Texture2d>),
    pub zoom: (imgui::TextureId, Rc<texture::Texture2d>),
}

impl Icons {
    pub fn from_display(display: &glium::Display, renderer: &mut ImguiRenderer) -> Self {
        // &* needed as these are lazy_static!'s
        let ret = Self {
            pan: load_icon(display, renderer, PAN_ICON_IMAGE.clone(), "pan"),
            pen: load_icon(display, renderer, PEN_ICON_IMAGE.clone(), "pan"),
            select: load_icon(display, renderer, SELECT_ICON_IMAGE.clone(), "select"),
            zoom: load_icon(display, renderer, ZOOM_ICON_IMAGE.clone(), "zoom"),
        };
        debug!("Loaded icons: {:?}", &ret);
        assert!(ret.pan.1.get_id() > 0);
        assert!(ret.pen.1.get_id() > 0);
        assert!(ret.select.1.get_id() > 0);
        assert!(ret.zoom.1.get_id() > 0);
        ret
    }
}

pub fn load_icon(
    display: &glium::Display,
    renderer: &mut ImguiRenderer,
    icon: data::SvgImageData,
    name: &'static str,
) -> (imgui::TextureId, Rc<texture::Texture2d>) {
    let (width, height, image) = icon;
    let slice = image.as_slice();
    let rawimage = texture::RawImage2d::from_raw_rgba(image, (width, height));
    let texture = Rc::new(texture::Texture2d::new(display, rawimage).expect(&format!(
        "Failed to upload texture for icon {} to GPU",
        name
    )));
    let id = renderer.textures().insert(texture.clone());
    (id, texture)
}
