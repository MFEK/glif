use std::cell::RefCell;

use crate::{
    editor::Editor,
    tools::ToolEnum,
    user_interface::gui::{
        layer_list::build_and_check_layer_list, prompts::build_and_check_prompts,
    },
};

use super::{icons, Interface};

// These are before transformation by STATE.dpi (glutin scale_factor)
pub const TOOLBOX_OFFSET_X: f32 = 10.;
pub const TOOLBOX_OFFSET_Y: f32 = TOOLBOX_OFFSET_X;
pub const TOOLBOX_WIDTH: f32 = 52.;

use enum_unitary::Bounded as _;
use lazy_static::lazy_static;
lazy_static! {
    pub static ref NUM_TOOLS: usize = ToolEnum::max_value() as usize;
    pub static ref TOOLBOX_HEIGHT: f32 = (*NUM_TOOLS * 34) as f32;
}

pub const LAYERBOX_WIDTH: f32 = 250.;
pub const LAYERBOX_HEIGHT: f32 = 250.;

use imgui::{self, Context, DrawData, FontId};
use imgui_sdl2::ImguiSdl2;
use imgui_skia_renderer::Renderer;
use sdl2::{event::Event, mouse::MouseState, video::Window};

pub mod layer_list;
pub mod prompts;

pub struct ImguiManager {
    pub imgui_context: Context,
    pub imgui_renderer: imgui_skia_renderer::Renderer,
    pub imgui_sdl2: imgui_sdl2::ImguiSdl2,
}

impl ImguiManager {
    pub fn new(window: &Window) -> Self {
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

        static ICON_FONT_TTF_DATA: &[u8] = include_bytes!("../../../resources/fonts/icons.ttf");

        let id = imgui.fonts().add_font(&[
            imgui::FontSource::TtfData {
                data: &crate::system_fonts::SYSTEMSANS.data,
                size_pixels: font_size,
                config: Some(imgui::FontConfig {
                    oversample_h: 3,
                    oversample_v: 3,
                    glyph_ranges: imgui::FontGlyphRanges::from_slice(&[
                        0x0020 as u16,
                        0x00FF as u16,
                        0x03B8 as u16, // for Greek theta
                        0x03B8 as u16, // for Greek theta
                        0,
                    ]),
                    ..Default::default()
                }),
            },
            imgui::FontSource::TtfData {
                data: ICON_FONT_TTF_DATA,
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

        let id1 = imgui.fonts().add_font(&[
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
                data: ICON_FONT_TTF_DATA,
                size_pixels: 28.,
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

        FONT_IDS.with(|ids| {
            ids.borrow_mut().push(id);
            ids.borrow_mut().push(id1);
        });
        PROMPT_STR.with(|prompt_str| prompt_str.borrow_mut().reserve(256));

        let imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);
        let imgui_renderer = Renderer::new(&mut imgui);
        return ImguiManager {
            imgui_context: imgui,
            imgui_renderer,
            imgui_sdl2,
        };
    }

    pub fn handle_imgui_event(&mut self, sdl_event: &Event) -> bool {
        self.imgui_sdl2
            .handle_event(&mut self.imgui_context, &sdl_event);
        if self.imgui_sdl2.ignore_event(sdl_event) {
            return true;
        };

        false
    }
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

pub fn build_imgui_ui<'ui>(
    context: &'ui mut Context,
    imsdl2: &mut ImguiSdl2,
    v: &mut Editor,
    i: &mut Interface,
    mouse_state: &MouseState,
) -> &'ui DrawData {
    imsdl2.prepare_frame(context.io_mut(), &i.sdl_window, mouse_state);
    let mut ui = context.frame();

    imgui::Window::new(imgui::im_str!("Tools"))
        .bg_alpha(1.) // See comment on fn redraw_skia
        .flags(
            imgui::WindowFlags::NO_RESIZE
                | imgui::WindowFlags::NO_MOVE
                | imgui::WindowFlags::NO_COLLAPSE,
        )
        .position(
            [TOOLBOX_OFFSET_X, TOOLBOX_OFFSET_Y],
            imgui::Condition::Always,
        )
        .size(
            [TOOLBOX_WIDTH, *TOOLBOX_HEIGHT + 90.],
            imgui::Condition::Always,
        )
        .build(&ui, || {
            build_and_check_button(v, &ui, ToolEnum::Pan, &icons::PAN);
            build_and_check_button(v, &ui, ToolEnum::Select, &icons::SELECT);
            ui.separator();
            build_and_check_button(v, &ui, ToolEnum::Zoom, &icons::ZOOM);
            build_and_check_button(v, &ui, ToolEnum::Measure, &icons::MEASURE);
            ui.separator();
            build_and_check_button(v, &ui, ToolEnum::Anchors, &icons::ANCHOR);
            ui.separator();
            build_and_check_button(v, &ui, ToolEnum::Pen, &icons::PEN);
            build_and_check_button(v, &ui, ToolEnum::Grid, &icons::GRID);
            build_and_check_button(v, &ui, ToolEnum::VWS, &icons::VWS);
            build_and_check_button(v, &ui, ToolEnum::PAP, &icons::_PAP);
            build_and_check_button(v, &ui, ToolEnum::Shapes, &icons::SHAPES);
            build_and_check_button(v, &ui, ToolEnum::Image, &icons::IMAGES);
            build_and_check_button(v, &ui, ToolEnum::Guidelines, &icons::GUIDELINES);
        });

    imgui::Window::new(imgui::im_str!("Layers"))
        .bg_alpha(1.)
        .flags(
            imgui::WindowFlags::NO_RESIZE
                | imgui::WindowFlags::NO_MOVE
                | imgui::WindowFlags::NO_COLLAPSE,
        )
        .position(
            [
                i.viewport.winsize.0 as f32 - LAYERBOX_WIDTH - TOOLBOX_OFFSET_X,
                i.viewport.winsize.1 as f32 - TOOLBOX_OFFSET_Y - LAYERBOX_HEIGHT,
            ],
            imgui::Condition::Always,
        )
        .size([LAYERBOX_WIDTH, LAYERBOX_HEIGHT], imgui::Condition::Always)
        .build(&ui, || build_and_check_layer_list(v, i, &ui));

    build_and_check_prompts(v, i, &mut ui);

    v.dispatch_tool_ui(i, &mut ui);

    imsdl2.prepare_render(&ui, &i.sdl_window);
    return ui.render();
}

thread_local! { pub static PROMPT_STR: RefCell<imgui::ImString> = RefCell::new(imgui::ImString::new("")); }
thread_local! { pub static PROMPT_CLR: RefCell<[f32; 4]> = RefCell::new([0., 0., 0., 1.]); }
thread_local! { pub static FONT_IDS: RefCell<Vec<FontId>> = RefCell::new(vec!()); }
