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
#[macro_use]
extern crate lazy_static;
extern crate backtrace;
extern crate clap;
extern crate colored;
extern crate derive_more;
extern crate enum_iterator;
#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate git_version; // for util::parse_args
extern crate font_kit;

extern crate imgui_winit_support;
extern crate skulpin;
extern crate skulpin_plugin_imgui;

extern crate clipboard;
extern crate regex;

// Our crates
extern crate glifparser;
extern crate mfek_ipc;

use crate::winit::dpi::LogicalSize;
use crate::winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use crate::winit::event_loop::{ControlFlow, EventLoop};
use skulpin::Window as _;
pub use skulpin::{skia_safe, winit};

use crate::skia_safe::{Contains, Point};

use enum_iterator::IntoEnumIterator as _;

use std::env;
use std::time::Instant;

pub use skulpin_plugin_imgui::imgui as imgui_rs;
pub use skulpin_plugin_imgui::{imgui::Ui as ImguiUi, ImguiRendererPlugin};

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
mod imgui;
mod io;
mod ipc;
mod renderer;
mod system_fonts;

use crate::renderer::constants::*;

fn main() {
    #[cfg(target_family = "windows")]
    util::set_codepage_utf8();

    // Set log level to WARN by default, useful for glifparser warnings.
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "warn");
    }
    env_logger::init();
    util::set_panic_hook();

    let window_size = (WIDTH, HEIGHT);
    let args = util::argparser::parse_args();
    let filename = filedialog::filename_or_panic(&args.filename, Some("glif"), None);
    let _glif = io::load_glif(&filename);

    if mfek_ipc::module_available("MFEKmetadata".into()) == mfek_ipc::Available::Yes {
        ipc::fetch_metrics();
    }

    let event_loop = EventLoop::new();

    let winit_window = winit::window::WindowBuilder::new()
        .with_title(format!(
            "Modular Font Editor K — Glyph editor — {}",
            filename.to_str().expect("Filename encoding erroneous")
        ))
        .with_inner_size(LogicalSize::new(window_size.0 as f64, window_size.1 as f64))
        .with_resizable(true)
        .build(&event_loop)
        .expect("Failed to create window");

    STATE.with(|v| {
        v.borrow_mut().winsize = winit_window.inner_size();
    });

    let imgui_manager = imgui::support::init_imgui_manager(&winit_window);
    imgui_manager.begin_frame(&winit_window);

    let window = skulpin::WinitWindow::new(&winit_window);

    let mut imgui_plugin = None;
    imgui_manager.with_context(|context| {
        imgui_plugin = Some(Box::new(ImguiRendererPlugin::new(context)));
    });

    // Create the renderer, which will draw to the window
    let renderer = skulpin::RendererBuilder::new()
        .prefer_fifo_present_mode()
        .use_vulkan_debug_layer(false)
        .coordinate_system(skulpin::CoordinateSystem::Logical)
        .add_plugin(imgui_plugin.unwrap())
        .build(&window);

    // Check if there were error setting up vulkan
    if let Err(e) = renderer {
        println!("Error during renderer construction: {:?}", e);
        return;
    }

    let mut renderer = renderer.unwrap();

    let _last_frame = Instant::now();

    STATE.with(|v| {
        v.borrow_mut().dpi = window.scale_factor();
    });

    let mut frame_count = 0;
    let mut frame = 0;
    let _was_resized = false;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        debug_event!("{:?}", event);
        if STATE.with(|v|v.borrow().quit_requested) {
            *control_flow = ControlFlow::Exit;
        }

        // Without this, the program will crash if it launches with the cursor over the window, as
        // the mouse event occurs before the redraw, which means that it uses an uninitialized
        // renderer. So we do this to assure first frame is drawn by RedrawRequested.
        match event {
            Event::RedrawRequested { .. } => {}
            _ => {
                if frame == 0 {
                    return;
                }
            }
        }

        let window = skulpin::WinitWindow::new(&winit_window);
        imgui_manager.handle_event(&winit_window, &event);
        // ImGui "events" don't really exist, click state etc. queried at time of drawing. It's an
        // immediate mode GUI. We use our events to update our Skia canvas.
        #[allow(deprecated)]
        match event {
            Event::LoopDestroyed => {}
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Q),
                            ..
                        },
                    ..
                } => {
                    if !CONSOLE.with(|c| c.borrow().active) {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode,
                            modifiers,
                            state: kstate,
                            ..
                        },
                    ..
                } => {
                    if kstate != ElementState::Pressed {
                        return;
                    }

                    if let Some(vk) = virtual_keycode {
                        events::console::set_state(vk, modifiers);
                    }

                    // We write to the Console in ReceivedCharacter, not here.
                    if CONSOLE.with(|c| {
                        if c.borrow().active {
                            if let Some(VirtualKeyCode::V) = virtual_keycode {
                                if modifiers.ctrl() {
                                    c.borrow_mut().handle_clipboard();
                                }
                            }
                        }
                        c.borrow().active
                    }) { return };

                    STATE.with(|v| {
                        let mode = v.borrow().mode;
                        let mut newmode = mode;
                        let mut scale = v.borrow().factor;
                        let mut offset = v.borrow().offset;
                        match virtual_keycode {
                            // Scales
                            Some(VirtualKeyCode::Key1) => scale = 1.,
                            Some(VirtualKeyCode::Equals) => {
                                scale = events::zoom_in_factor(scale, &v);
                            }
                            Some(VirtualKeyCode::Minus) => {
                                scale = events::zoom_out_factor(scale, &v);
                            }
                            // Translations
                            Some(VirtualKeyCode::Up) => {
                                offset.1 += OFFSET_FACTOR;
                            }
                            Some(VirtualKeyCode::Down) => {
                                offset.1 += -OFFSET_FACTOR;
                            }
                            Some(VirtualKeyCode::Left) => {
                                offset.0 += OFFSET_FACTOR;
                            }
                            Some(VirtualKeyCode::Right) => {
                                offset.0 += -OFFSET_FACTOR;
                            }
                            // Modes
                            Some(VirtualKeyCode::A) => {
                                newmode = state::Mode::Pan;
                            }
                            Some(VirtualKeyCode::P) => {
                                newmode = state::Mode::Pen;
                            }
                            Some(VirtualKeyCode::V) => {
                                newmode = state::Mode::Select;
                            }
                            Some(VirtualKeyCode::Z) => {
                                newmode = state::Mode::Zoom;
                            }
                            // Toggles: trigger_toggle_on defined in events module
                            Some(VirtualKeyCode::Key3) => {
                                trigger_toggle_on!(v, point_labels, PointLabels, modifiers.shift());
                            }
                            Some(VirtualKeyCode::Grave) => {
                                #[rustfmt::skip]
                                trigger_toggle_on!(v, preview_mode, PreviewMode, !modifiers.shift());

                                trigger_toggle_on!(v, handle_style, HandleStyle, modifiers.shift());
                            }
                            _ => {}
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
                WindowEvent::ReceivedCharacter(ch) => {
                    if !CONSOLE.with(|c| c.borrow().active) {
                        return;
                    }
                    CONSOLE.with(|c| c.borrow_mut().handle_ch(ch));
                }
                WindowEvent::CursorMoved { position, .. } => {
                    STATE.with(|v| {
                        let mode = v.borrow().mode;

                        match mode {
                            #[rustfmt::skip]
                            state::Mode::Pan => events::pan::mouse_moved(position, &v),
                            state::Mode::Pen => events::pen::mouse_moved(position, &v),
                            state::Mode::Select => events::select::mouse_moved(position, &v),
                            state::Mode::Zoom => events::zoom::mouse_moved(position, &v),
                            _ => false,
                        };
                    });
                }
                WindowEvent::MouseInput {
                    state: mstate,
                    button,
                    modifiers,
                    ..
                } => {
                    STATE.with(|v| {
                        // Ignore events if we are clicking on Dear ImGui toolbox.
                        let toolbox_rect = imgui::toolbox_rect();
                        let absolute_position = v.borrow().absolute_mousepos;
                        if toolbox_rect.contains(Point::from((
                            absolute_position.x as f32,
                            absolute_position.y as f32,
                        ))) {
                            return;
                        }

                        let meta = events::MouseMeta{modifiers, button};

                        let mode = v.borrow().mode;
                        let position = v.borrow().mousepos;
                        v.borrow_mut().mousedown = mstate == ElementState::Pressed;

                        match mode {
                            state::Mode::Select => {
                                events::select::mouse_button(position, &v, meta)
                            }
                            _ => false,
                        };

                        match mstate {
                            ElementState::Pressed => {
                                match mode {
                                    state::Mode::Pen => {
                                        events::pen::mouse_pressed(position, &v, meta)
                                    }
                                    state::Mode::Select => {
                                        events::select::mouse_pressed(position, &v, meta)
                                    }
                                    _ => false,
                                };
                            }
                            ElementState::Released => {
                                match mode {
                                    state::Mode::Pen => {
                                        events::pen::mouse_released(position, &v, meta)
                                    }
                                    state::Mode::Select => {
                                        events::select::mouse_released(position, &v, meta)
                                    }
                                    state::Mode::Zoom => {
                                        events::zoom::mouse_released(position, &v, meta);
                                        events::center_cursor(&winit_window).is_ok()
                                    }
                                    _ => false,
                                };
                            }
                        }
                    });
                }
                WindowEvent::Resized(size) => {
                    STATE.with(|v| {
                        v.borrow_mut().winsize = size;
                    });
                }
                _ => (),
            },
            Event::RedrawRequested { .. } => {
                if let Err(e) = renderer.draw(&window, |canvas, _coordinate_system_helper| {
                    imgui_manager.begin_frame(&winit_window);
                    frame_count += 1;

                    renderer::render_frame(canvas);

                    {
                        imgui_manager.with_ui(|ui: &mut ImguiUi| {
                            imgui::build_imgui_ui(ui);
                        });
                    }

                    imgui_manager.render(&winit_window);
                }) {
                    println!("Error during draw: {:?}", e);
                    *control_flow = winit::event_loop::ControlFlow::Exit
                }
            }
            Event::MainEventsCleared => {
                winit_window.request_redraw();
            }
            _ => return,
        }
        frame += 1;
    });
}
