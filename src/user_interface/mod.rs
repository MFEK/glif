use events::ToolEnum;
use imgui::{self, Context};

use crate::{events, state::Editor};

pub mod icons;

// These are before transformation by STATE.dpi (glutin scale_factor)
const TOOLBOX_OFFSET_X: f32 = 10.;
const TOOLBOX_OFFSET_Y: f32 = TOOLBOX_OFFSET_X;
const TOOLBOX_WIDTH: f32 = 55.;
const TOOLBOX_HEIGHT: f32 = 220.;

pub fn setup_imgui() -> Context {
    let mut imgui = Context::create();
    {
        // Fix incorrect colors with sRGB framebuffer
        fn imgui_gamma_to_linear(col: [f32; 4]) -> [f32; 4] {
            let x = col[0].powf(2.2);
            let y = col[1].powf(2.2);
            let z = col[2].powf(2.2);
            let w = 1.0 - (1.0 - col[3]).powf(2.2);
            [x, y, z, w]
        }

        let style = imgui.style_mut();
        for col in 0..style.colors.len() {
            style.colors[col] = imgui_gamma_to_linear(style.colors[col]);
        }
    }

    imgui.set_ini_filename(None);
    imgui.style_mut().use_light_colors();

    // TODO: Implement proper DPI scaling
    let scale_factor = 1.;
    let font_size = (16.0 * scale_factor) as f32;
    let icon_font_size = (36.0 * scale_factor) as f32;

    imgui.fonts().add_font(&[
        imgui::FontSource::TtfData {
            data: &crate::system_fonts::SYSTEMSANS.data,
            size_pixels: font_size,
            config: Some(imgui::FontConfig {
                oversample_h: 3,
                oversample_v: 3,
                ..Default::default()
            }),
        },
        imgui::FontSource::TtfData {
            data: include_bytes!("../../resources/fonts/icons.ttf"),
            size_pixels: icon_font_size,
            config: Some(imgui::FontConfig {
                glyph_ranges: imgui::FontGlyphRanges::from_slice(&[
                    0xF000 as u16,
                    0xF100 as u16,
                    0,
                ]),
                ..Default::default()
            }),
        },
    ]);

    imgui
}

pub fn build_and_check_button(v: &mut Editor, ui: &imgui::Ui, mode: ToolEnum, icon: &[u8]) {
    let mut pop_me = None;
    if v.get_tool() == mode {
        pop_me = Some(ui.push_style_color(imgui::StyleColor::Button, [0., 0., 0., 0.2]));
    }
    // Icons are always constant so this is not really unsafe.
    ui.button(
        unsafe { imgui::ImStr::from_utf8_with_nul_unchecked(icon) },
        [0., 30.],
    );
    if ui.is_item_clicked(imgui::MouseButton::Left) {
        v.set_tool(mode);
    }
    if let Some(p) = pop_me {
        p.pop(ui);
    }
}

pub fn build_imgui_ui(v: &mut Editor, ui: &mut imgui::Ui) {
    let mode = v.get_tool();

    imgui::Window::new(imgui::im_str!("Tools"))
        .bg_alpha(1.) // See comment on fn redraw_skia
        .flags(
            #[rustfmt::skip]
                    imgui::WindowFlags::NO_RESIZE
                | imgui::WindowFlags::NO_MOVE
                | imgui::WindowFlags::NO_COLLAPSE,
        )
        .position(
            [TOOLBOX_OFFSET_X, TOOLBOX_OFFSET_Y],
            imgui::Condition::Always,
        )
        .size([TOOLBOX_WIDTH, TOOLBOX_HEIGHT], imgui::Condition::Always)
        .build(ui, || {
            build_and_check_button(v, &ui, ToolEnum::Pan, &icons::PAN);
            build_and_check_button(v, &ui, ToolEnum::Select, &icons::SELECT);
            ui.separator();
            build_and_check_button(v, &ui, ToolEnum::Zoom, &icons::ZOOM);
            ui.separator();
            build_and_check_button(v, &ui, ToolEnum::Pen, &icons::PEN);
            build_and_check_button(v, &ui, ToolEnum::VWS, &icons::VWS);
        });

    //TODO: Add UI event dispatch here.
}
