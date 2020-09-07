use glium;
use glium::texture;
use glium::GlObject;
use imgui;
use imgui_glium_renderer::Renderer as ImguiRenderer;

use std::borrow::Borrow;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use std::{rc::Rc, sync::Arc};

use crate::events;
use crate::state::Mode;
use crate::STATE;

pub mod icons;

pub use self::icons::set_icons;

use self::icons::Icons;

// These are before transformation by STATE.dpi (glutin scale_factor)
const TOOLBOX_OFFSET_X: f32 = 10.;
const TOOLBOX_OFFSET_Y: f32 = TOOLBOX_OFFSET_X;
const TOOLBOX_WIDTH: f32 = 45.;
const TOOLBOX_HEIGHT: f32 = 220.;

fn button_from_name(name: &'static str) -> imgui::ImageButton {
    let (imgui_tex, gl_tex) = STATE.with(|v| {
        v.borrow()
            .icons
            .as_ref()
            .unwrap()
            .get(name)
            .unwrap()
            .clone()
    });
    imgui::ImageButton::new(imgui_tex, [gl_tex.width() as f32, gl_tex.height() as f32])
        .frame_padding(3)
}

pub fn build_imgui_ui(ui: &mut imgui::Ui) {
    // These clones are "free" since it's an Rc<_>
    let mut pan_button = button_from_name("pan");
    let mut pen_button = button_from_name("pen");
    let mut select_button = button_from_name("select");
    let mut zoom_button = button_from_name("zoom");

    STATE.with(|v| {
        let mode = v.borrow().mode;

        match v.borrow().mode {
            Mode::Pan => {
                pan_button = pan_button.background_col([0., 0., 0., 0.2]);
            }
            Mode::Pen => {
                pen_button = pen_button.background_col([0., 0., 0., 0.2]);
            }
            Mode::Select => {
                select_button = select_button.background_col([0., 0., 0., 0.2]);
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
            .position(
                [TOOLBOX_OFFSET_X, TOOLBOX_OFFSET_Y],
                imgui::Condition::Always,
            )
            .size([TOOLBOX_WIDTH, TOOLBOX_HEIGHT], imgui::Condition::Always)
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
                ui.separator();
                pen_button.build(ui);
                if ui.is_item_clicked(imgui::MouseButton::Left) {
                    v.borrow_mut().mode = Mode::Pen;
                }
            });

        let new_mode = v.borrow().mode;
        if new_mode != mode {
            events::mode_switched(mode, new_mode);
        }
    });
}

use font_kit::{
    family_name::FamilyName as FKFamilyName, handle::Handle as FKHandle, properties::Properties,
    source::SystemSource,
};

struct Font {
    data: Vec<u8>,
    path: Option<PathBuf>,
}

fn load_font(family: &[FKFamilyName]) -> Font {
    let source = SystemSource::new();
    let font = source
        .select_best_match(family, &Properties::new())
        .unwrap();
    match font {
        FKHandle::Path { path, .. } => Font {
            path: Some(path.clone()),
            data: fs::read(path).expect("Failed to open font path system specified"),
        },
        FKHandle::Memory { bytes, .. } => Font {
            path: None,
            data: Arc::try_unwrap(bytes).expect("Failed to load in-memory font"),
        },
    }
}

lazy_static! {
    static ref SYSTEMSANS: Font = load_font(&[
        FKFamilyName::Title("Segoe UI".to_string()),
        FKFamilyName::SansSerif
    ]);
    static ref SYSTEMMONO: Font = load_font(&[FKFamilyName::Monospace]);
}

pub fn set_imgui_fonts(imgui: &mut imgui::Context) {
    let dpi = STATE.with(|v| v.borrow().dpi as f32);
    let mut fontconfig = imgui::FontConfig::default();
    fontconfig.oversample_h = f32::ceil(dpi) as i32 + 1;
    fontconfig.oversample_v = fontconfig.oversample_h;
    imgui.fonts().add_font(&[imgui::FontSource::TtfData {
        data: &SYSTEMSANS.data,
        size_pixels: 14.,
        config: Some(fontconfig),
    }]);
}

pub fn set_imgui_dpi(imgui: &mut imgui::Context, window_size: (u32, u32)) {
    STATE.with(|v| {
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

use reclutch::skia::Rect;

pub fn toolbox_rect() -> Rect {
    let dpi = STATE.with(|v| v.borrow().dpi) as f32;
    Rect::from_point_and_size(
        (TOOLBOX_OFFSET_X * dpi, TOOLBOX_OFFSET_Y * dpi),
        (TOOLBOX_WIDTH * dpi, TOOLBOX_HEIGHT * dpi),
    )
}
