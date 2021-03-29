//! MFEKglif - A cross-platform .glif renderer and editor.
//! Main author is Fredrick Brennan (@ctrlcctrlv); see AUTHORS.
//! (c) 2020. Apache 2.0 licensed.
#![allow(non_snake_case)] // for our name MFEKglif
#![feature(
    assoc_char_funcs,
    panic_info_message,
    stmt_expr_attributes,
    cell_leak,
    or_patterns
)]

// Cargo.toml comments say what crates are used for what.
extern crate backtrace;
extern crate clap;
extern crate colored;
extern crate derive_more;
extern crate enum_iterator;
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate font_kit;
extern crate git_version; // for util::parse_args

extern crate skulpin;

extern crate clipboard;
extern crate regex;

// Our crates
extern crate glifparser;
extern crate mfek_ipc;
extern crate xmltree;

extern crate sdl2;

use command::{Command, CommandInfo};

//use renderer::render_frame;
use sdl2::keyboard::Keycode;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Mod,
    surface::Surface,
    video::Window,
    Sdl,
};
pub use skulpin::skia_safe;
use skulpin::{rafx::api::RafxError, rafx::api::RafxExtents2D, LogicalSize, RendererBuilder};

use imgui_skia_renderer::Renderer;

use enum_iterator::IntoEnumIterator as _;

use std::collections::HashSet;

// Provides thread-local global variables.
pub mod state;
pub use crate::state::Glyph; // types
pub use crate::state::{HandleStyle, PointLabels, PreviewMode}; // enums
pub use crate::state::{CONSOLE, STATE, TOOL_DATA}; // globals

mod filedialog;
#[macro_use]
pub mod util;
#[macro_use]
mod events;
mod command;
mod io;
mod ipc;
mod renderer;
mod system_fonts;
mod user_interface;

use crate::renderer::constants::*;

struct WindowSettings {
    filename: String,
}

fn main() {
    env_logger::init();

    let args = util::argparser::parse_args();
    let filename = filedialog::filename_or_panic(&args.filename, Some("glif"), None);

    // Makes glyph available to on_load_glif events
    let _glif = io::load_glif(&filename);

    // events for on_load_glif go here
    events::vws::on_load_glif();

    if mfek_ipc::module_available("MFEKmetadata".into()) == mfek_ipc::Available::Yes {
        ipc::fetch_metrics();
    }

    let (sdl_context, window) = initialize_sdl(&WindowSettings {
        filename: filename.to_str().unwrap().to_string(),
    });

    // Skulpin initialization TODO: proper error handling
    let mut renderer = initialize_skulpin_renderer(&window).unwrap();

    // set up imgui
    let mut imgui = user_interface::setup_imgui();
    let mut imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);
    let imgui_renderer = Renderer::new(&mut imgui);

    let mut event_pump = sdl_context
        .event_pump()
        .expect("Could not create sdl event pump");

    command::initialize_keybinds();

    'main_loop: loop {
        // Create a set of pressed Keys.
        let keys_down: HashSet<Keycode> = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        // sdl event handling
        for event in event_pump.poll_iter() {
            imgui_sdl2.handle_event(&mut imgui, &event);
            if imgui_sdl2.ignore_event(&event) {
                continue;
            };

            // we're gonna handle some of these events before handling commands so that we don't have the logic for this stuff
            // intertwined in command handling
            match &event {
                Event::Quit { .. } => break 'main_loop,
                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    keymod: km,
                    ..
                } => {
                    if km.contains(Mod::LCTRLMOD) || km.contains(Mod::RCTRLMOD) {
                        break 'main_loop;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    keymod: km,
                    ..
                } => {
                    if km.contains(Mod::LSHIFTMOD) || km.contains(Mod::RSHIFTMOD) {
                        STATE.with(|v| {
                            io::save_glif(v);
                        });
                        continue;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::E),
                    keymod: km,
                    ..
                } => {
                    if km.contains(Mod::LSHIFTMOD) || km.contains(Mod::RSHIFTMOD) {
                        STATE.with(|v| {
                            io::export_glif(v);
                        });
                        continue;
                    }
                }

                // we're gonna handle console text input here too as this should steal input from the command system
                Event::TextInput { text, .. } => {
                    if CONSOLE.with(|c| return c.borrow_mut().active) {
                        for ch in text.chars() {
                            CONSOLE.with(|c| c.borrow_mut().handle_ch(ch));
                        }
                        continue;
                    }
                }
                _ => {}
            }

            match event {
                Event::KeyDown {
                    keycode, keymod, ..
                } => {
                    // we don't care about keydown events that have no keycode
                    if !keycode.is_some() {
                        continue;
                    }
                    let keycode = keycode.unwrap();

                    events::console::set_state(keycode, keymod);
                    if CONSOLE.with(|c| c.borrow_mut().active) {
                        continue;
                    }

                    STATE.with(|v| {
                        let mode = v.borrow().mode;
                        let mut newmode = mode;
                        let mut scale = v.borrow().factor;
                        let mut offset = v.borrow().offset;

                        // check if we've got a command
                        let command_info: Option<CommandInfo> =
                            command::keycode_to_command(&keycode, &keys_down);
                        if !command_info.is_some() {
                            return;
                        }
                        let command_info = command_info.unwrap();

                        match command_info.command {
                            Command::ResetScale => {
                                scale = 1.;
                            }
                            Command::ZoomIn => {
                                scale = events::zoom_in_factor(scale, &v);
                            }
                            Command::ZoomOut => {
                                scale = events::zoom_out_factor(scale, &v);
                            }
                            Command::NudgeUp => {
                                offset.1 += OFFSET_FACTOR;
                            }
                            Command::NudgeDown => {
                                offset.1 -= OFFSET_FACTOR;
                            }
                            Command::NudgeLeft => {
                                offset.0 += OFFSET_FACTOR;
                            }
                            Command::NudgeRight => {
                                offset.0 -= OFFSET_FACTOR;
                            }
                            Command::ToolPan => {
                                newmode = state::Mode::Pan;
                            }
                            Command::ToolPen => {
                                newmode = state::Mode::Pen;
                            }
                            Command::ToolSelect => {
                                newmode = state::Mode::Select;
                            }
                            Command::ToolZoom => {
                                newmode = state::Mode::Zoom;
                            }
                            Command::ToolVWS => {
                                newmode = state::Mode::VWS;
                            }
                            Command::TogglePointLabels => {
                                trigger_toggle_on!(
                                    v,
                                    point_labels,
                                    PointLabels,
                                    command_info.command_mod.shift
                                );
                            }
                            Command::TogglePreviewMode => {
                                trigger_toggle_on!(
                                    v,
                                    preview_mode,
                                    PreviewMode,
                                    !command_info.command_mod.shift
                                );
                            }
                            Command::ToggleConsole => {
                                CONSOLE.with(|c| {
                                    c.borrow_mut().active = true;
                                });
                            }

                            _ => unreachable!(
                                "The remaining Command enums should never be returned."
                            ),
                        }

                        if mode != newmode {
                            v.borrow_mut().mode = newmode;
                            events::mode_switched(mode, newmode);

                            debug!(
                                "Scale factor now {}; offset {:+}{:+}; mode {:?}",
                                v.borrow().factor,
                                v.borrow().offset.0,
                                v.borrow().offset.1,
                                v.borrow().mode
                            );
                        }

                        v.borrow_mut().offset = offset;
                        v.borrow_mut().factor = scale;
                    });
                }

                Event::MouseMotion { x, y, .. } => {
                    let position = (x as f64, y as f64);
                    STATE.with(|v| {
                        let mode = v.borrow().mode;

                        match mode {
                            #[rustfmt::skip]
                            state::Mode::Pan => events::pan::mouse_moved(position, &v),
                            state::Mode::Pen => events::pen::mouse_moved(position, &v),
                            state::Mode::Select => {
                                events::select::mouse_moved(position, &v);
                                events::vws::update_previews(position, &v)
                            }
                            state::Mode::VWS => events::vws::mouse_moved(position, &v),
                            state::Mode::Zoom => events::zoom::mouse_moved(position, &v),
                        };
                    });
                }

                Event::MouseButtonDown { mouse_btn, .. } => {
                    STATE.with(|v| {
                        let keymod = command::key_down_to_mod(&keys_down);
                        let meta = events::MouseMeta {
                            button: mouse_btn,
                            modifiers: keymod,
                        };

                        let mode = v.borrow().mode;
                        let position = v.borrow().mousepos;
                        v.borrow_mut().mousedown = true;

                        match mode {
                            state::Mode::Select => events::select::mouse_button(position, &v, meta),
                            state::Mode::VWS => events::vws::mouse_button(position, &v, meta),
                            _ => false,
                        };

                        match mode {
                            state::Mode::Pen => events::pen::mouse_pressed(position, &v, meta),
                            state::Mode::Select => {
                                events::select::mouse_pressed(position, &v, meta)
                            }
                            state::Mode::VWS => events::vws::mouse_pressed(position, &v, meta),
                            _ => false,
                        };
                    });
                }

                Event::MouseButtonUp { mouse_btn, .. } => {
                    STATE.with(|v| {
                        let keymod = command::key_down_to_mod(&keys_down);
                        let meta = events::MouseMeta {
                            button: mouse_btn,
                            modifiers: keymod,
                        };

                        let mode = v.borrow().mode;
                        let position = v.borrow().mousepos;
                        v.borrow_mut().mousedown = false;

                        match mode {
                            state::Mode::Pen => events::pen::mouse_released(position, &v, meta),
                            state::Mode::Select => {
                                events::select::mouse_released(position, &v, meta)
                            }
                            state::Mode::Zoom => {
                                events::zoom::mouse_released(position, &v, meta);
                                events::center_cursor(&sdl_context, &window);
                                true
                            }
                            state::Mode::VWS => events::vws::mouse_released(position, &v, meta),
                            _ => false,
                        };
                    });
                }

                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::Resized(x, y) => {
                        STATE.with(|v| {
                            v.borrow_mut().winsize = (x as u32, y as u32);
                        });
                    }

                    _ => {}
                },
                _ => {}
            }
        }

        // build and render imgui
        imgui_sdl2.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());
        let mut ui = imgui.frame();
        user_interface::build_imgui_ui(&mut ui);

        imgui_sdl2.prepare_render(&ui, &window);
        let dd = ui.render();

        // draw glyph preview and imgui with skia
        let (window_width, window_height) = window.vulkan_drawable_size();
        let extents = RafxExtents2D {
            width: window_width,
            height: window_height,
        };

        let drew = renderer.draw(extents, 1.0, |canvas, _coordinate_system_helper| {
            renderer::render_frame(canvas);
            imgui_renderer.render_imgui(canvas, dd);
        });

        if drew.is_err() {
            warn!("Failed to draw frame. This can happen when resizing due to VkError(ERROR_DEVICE_LOST); if happens otherwise, file an issue.");
        }
    }
}

fn initialize_sdl(ws: &WindowSettings) -> (Sdl, Window) {
    // SDL initialization
    let sdl_context = sdl2::init().expect("Failed to initialize sdl2");
    let video_subsystem = sdl_context
        .video()
        .expect("Failed to create sdl video subsystem");

    video_subsystem.text_input().start();

    let logical_size = LogicalSize {
        width: WIDTH,
        height: HEIGHT,
    };

    STATE.with(|v| {
        v.borrow_mut().winsize = (WIDTH as u32, HEIGHT as u32);
    });

    let mut window = video_subsystem
        .window(
            &format!("MFEKglif — {}", ws.filename),
            logical_size.width,
            logical_size.height,
        )
        .position_centered()
        .allow_highdpi()
        .vulkan()
        .resizable()
        .build()
        .expect("Failed to create window");

    let logo = include_bytes!("../doc/logo.png");
    let im = image::load_from_memory_with_format(logo, image::ImageFormat::Png)
        .unwrap()
        .into_rgb8();
    let mut bytes = im.into_vec();
    let surface = Surface::from_data(
        &mut bytes,
        701,
        701,
        701 * 3,
        sdl2::pixels::PixelFormatEnum::RGB888,
    )
    .unwrap();
    window.set_icon(surface);

    (sdl_context, window)
}

fn initialize_skulpin_renderer(sdl_window: &Window) -> Result<skulpin::Renderer, RafxError> {
    let (window_width, window_height) = sdl_window.vulkan_drawable_size();

    let extents = RafxExtents2D {
        width: window_width,
        height: window_height,
    };

    let scale_to_fit = skulpin::skia_safe::matrix::ScaleToFit::Start;
    let visible_range = skulpin::skia_safe::Rect {
        left: 0.0,
        right: WIDTH as f32,
        top: 0.0,
        bottom: HEIGHT as f32,
    };

    let renderer = RendererBuilder::new()
        .coordinate_system(skulpin::CoordinateSystem::VisibleRange(
            visible_range,
            scale_to_fit,
        ))
        .build(sdl_window, extents);

    return renderer;
}
