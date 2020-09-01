use glium;
use glium::texture;
use glium::GlObject;
use imgui;
use imgui_glium_renderer::Renderer as ImguiRenderer;

use std::borrow::Borrow;
use std::rc::Rc;
use std::time::Instant;

use crate::state::{state, Icons, Mode};

mod icons;

fn button_from_texture(tex: (imgui::TextureId, Rc<texture::Texture2d>)) -> imgui::ImageButton {
    imgui::ImageButton::new(tex.0, [tex.1.width() as f32, tex.1.height() as f32]).frame_padding(3)
}

pub fn build_imgui_ui(ui: &mut imgui::Ui) {
    state.with(|v| {
        //let select_button = button_from_texture(&v.borrow().icons.as_ref().unwrap().select);
        let select_ref = v.borrow().icons.as_ref().unwrap().select.clone();
        let mut select_button = button_from_texture(select_ref);
        let pan_ref = v.borrow().icons.as_ref().unwrap().pan.clone();
        let mut pan_button = button_from_texture(pan_ref);
        let zoom_ref = v.borrow().icons.as_ref().unwrap().zoom.clone();
        let mut zoom_button = button_from_texture(zoom_ref);

        match v.borrow().mode {
            Mode::Select => {
                select_button = select_button.background_col([0., 0., 0., 0.1]);
            }
            Mode::Pan => {
                pan_button = pan_button.background_col([0., 0., 0., 0.2]);
            }
            Mode::Zoom => {
                zoom_button = zoom_button.background_col([0., 0., 0., 0.2]);
            }
        }

        imgui::Window::new(im_str!("Tools"))
            .bg_alpha(1.) // See comment on fn redraw_skia
            .flags(
                #[rustfmt::skip]
                      imgui::WindowFlags::NO_RESIZE
                    | imgui::WindowFlags::NO_MOVE
                    | imgui::WindowFlags::NO_COLLAPSE,
            )
            .position([10., 10.], imgui::Condition::Always)
            .size([50.0, 200.0], imgui::Condition::Always)
            .build(ui, || {
                select_button.build(ui);
                if ui.is_item_clicked(imgui::MouseButton::Left) {
                    v.borrow_mut().mode = Mode::Select;
                }
                pan_button.build(ui);
                if ui.is_item_clicked(imgui::MouseButton::Left) {
                    v.borrow_mut().mode = Mode::Pan;
                }
                ui.separator();
                zoom_button.build(ui);
                if ui.is_item_clicked(imgui::MouseButton::Left) {
                    v.borrow_mut().mode = Mode::Zoom;
                }
            });
    });
}

pub fn set_imgui_fonts(imgui: &mut imgui::Context) {
    let dpi = state.with(|v| v.borrow().dpi as f32);
    let mut fontconfig = imgui::FontConfig::default();
    fontconfig.oversample_h = f32::ceil(dpi) as i32 + 1;
    fontconfig.oversample_v = fontconfig.oversample_h;
    imgui.fonts().add_font(&[imgui::FontSource::TtfData {
        data: include_bytes!(concat!(
            env!("PWD"),
            "/",
            "resources/fonts/Ubuntu-Regular.ttf"
        )),
        size_pixels: 14.,
        config: Some(fontconfig),
    }]);
}

pub fn set_imgui_dpi(imgui: &mut imgui::Context, window_size: (u32, u32)) {
    state.with(|v| {
        let dpi = v.borrow().dpi as f32;
        imgui.style_mut().scale_all_sizes(dpi);
        imgui.io_mut().display_size = [
            window_size.0 as f32 * (1. / dpi),
            window_size.1 as f32 * (1. / dpi),
        ];
        imgui.io_mut().font_global_scale = 1.;
        imgui.style_mut().use_light_colors();
    });
}

pub fn render_imgui_frame(
    target: &mut glium::framebuffer::SimpleFrameBuffer,
    imgui: &mut imgui::Context,
    last_frame: &mut Instant,
    renderer: &mut ImguiRenderer,
) {
    let io = imgui.io_mut();

    *last_frame = io.update_delta_time(*last_frame);
    let mut ui = imgui.frame();
    build_imgui_ui(&mut ui);

    let draw_data = ui.render();
    renderer
        .render(target, draw_data)
        .expect("Rendering failed");
}
