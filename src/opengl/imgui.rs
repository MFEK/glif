use glium;
use imgui;
use imgui_glium_renderer::Renderer as ImguiRenderer;

use std::time::Instant;

use crate::state::state;

mod icons;

pub fn build_imgui_ui(ui: &mut imgui::Ui) {
    imgui::Window::new(im_str!("Hello world"))
        .bg_alpha(1.) // See comment on fn redraw_skia
        .size([300.0, 100.0], imgui::Condition::FirstUseEver)
        .build(ui, || {
            ui.text(im_str!("Hello world!"));
            ui.text(im_str!("This...is...imgui-rs!"));
            ui.separator();
            let mouse_pos = ui.io().mouse_pos;
            ui.text(format!(
                "Mouse Position: ({:.1},{:.1})",
                mouse_pos[0], mouse_pos[1]
            ));
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
