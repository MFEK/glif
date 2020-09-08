//! Qglif - A cross-platform .glif renderer and editor.
//! Main author is Fredrick Brennan (@ctrlcctrlv); see AUTHORS.
//! (c) 2020. Apache 2.0 licensed.
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
extern crate enum_iterator;
#[macro_use]
extern crate git_version; // for util::parse_args
extern crate font_kit;
#[macro_use]
extern crate skulpin;
#[macro_use]
extern crate skulpin_plugin_imgui;
extern crate imgui_winit_support;
// Our crates
extern crate glifparser;

use skulpin::Window as _;
pub use skulpin::{skia_safe, winit};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use skia_safe::{Contains, Point, Rect};

use enum_iterator::IntoEnumIterator;

use std::time::Instant;

#[macro_use]
pub use skulpin_plugin_imgui::{imgui::Ui as ImguiUi, ImguiRendererPlugin};
pub use skulpin_plugin_imgui::imgui as imgui_rs;

#[macro_use]
mod util;
mod io;
// Provides a thread-local global `STATE` variable
mod state;
use state::{Glyph, PointLabels};
pub use state::{PEN_DATA, STATE};
mod events;
mod imgui;
mod renderer;

use renderer::constants::*;

fn main() {
    util::set_panic_hook();

    let window_size = (WIDTH, HEIGHT);
    STATE.with(|v| {
        v.borrow_mut().winsize = window_size.into();
    });

    let args = util::argparser::parse_args();
    let filename = args.filename;
    let glif = io::load_glif(&filename);

    let event_loop = EventLoop::new();

    let winit_window = winit::window::WindowBuilder::new()
        .with_title(format!("Qglif: {}", filename))
        .with_inner_size(PhysicalSize::new(
            window_size.0 as f64,
            window_size.1 as f64,
        ))
        .with_resizable(true)
        .build(&event_loop)
        .expect("Failed to create window");

    let imgui_manager = imgui::support::init_imgui_manager(&winit_window);
    imgui_manager.begin_frame(&winit_window);

    let window = skulpin::WinitWindow::new(&winit_window);

    let mut imgui_plugin = None;
    imgui_manager.with_context(|context| {
        imgui_plugin = Some(Box::new(ImguiRendererPlugin::new(context)));
    });

    let scale_to_fit = skulpin::skia_safe::matrix::ScaleToFit::Center;
    let visible_range = skulpin::skia_safe::Rect {
        left: 0.0,
        right: window_size.0 as f32,
        top: 0.0,
        bottom: window_size.1 as f32,
    };

    // Create the renderer, which will draw to the window
    let renderer = skulpin::RendererBuilder::new()
        .use_vulkan_debug_layer(false)
        .coordinate_system(skulpin::CoordinateSystem::None)
        .add_plugin(imgui_plugin.unwrap())
        .build(&window);

    // Check if there were error setting up vulkan
    if let Err(e) = renderer {
        println!("Error during renderer construction: {:?}", e);
        return;
    }

    let mut renderer = renderer.unwrap();

    let mut last_frame = Instant::now();

    //imgui.set_ini_filename(None);

    STATE.with(|v| {
        v.borrow_mut().dpi = window.scale_factor();
    });

    let mut frame_count = 0;
    let mut frame = 0;
    let mut was_resized = false;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        debug_events!("{:?}", event);

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
                            virtual_keycode: Some(VirtualKeyCode::Escape | VirtualKeyCode::Q),
                            ..
                        },
                    ..
                }
                | WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
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
                            // Toggles
                            Some(VirtualKeyCode::Key3) => {
                                let point_labels = v.borrow().point_labels;
                                if modifiers.shift() {
                                    let mut e = PointLabels::into_enum_iter()
                                        .cycle()
                                        .skip(1 + point_labels as usize);
                                    let pl = e.next().unwrap();
                                    v.borrow_mut().point_labels = pl;
                                }
                            }
                            _ => {}
                        }
                        if mode != newmode {
                            v.borrow_mut().mode = newmode;
                            events::mode_switched(mode, newmode);
                        }

                        debug!(
                            "Scale factor now {}; offset {:?}; mode {:?}",
                            v.borrow().factor,
                            v.borrow().offset,
                            v.borrow().mode
                        );
                    });
                }
                WindowEvent::CursorMoved { position, .. } => {
                    renderer.draw(&window, |canvas, coordinate_system_helper| {
                        STATE.with(|v| {
                            let mode = v.borrow().mode;

                            match mode {
                                #[rustfmt::skip]
                                state::Mode::Pan => events::mouse_moved_move(position, &v, canvas),
                                state::Mode::Pen => events::mouse_moved_pen(position, &v, canvas),
                                state::Mode::Select => {
                                    events::mouse_moved_select(position, &v, canvas)
                                }
                                state::Mode::Zoom => events::mouse_moved_zoom(position, &v, canvas),
                                _ => false,
                            };
                        });
                        renderer::update_viewport(canvas);
                        renderer::render_frame(canvas);
                    });
                }
                WindowEvent::MouseInput {
                    state: mstate,
                    button,
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

                        renderer.draw(&window, |canvas, coordinate_system_helper| {
                            let mode = v.borrow().mode;
                            let position = v.borrow().mousepos;
                            v.borrow_mut().mousedown = mstate == ElementState::Pressed;

                            match mode {
                                state::Mode::Select => {
                                    events::mouse_button_select(position, &v, canvas, button)
                                }
                                _ => false,
                            };

                            match mstate {
                                ElementState::Pressed => {
                                    match mode {
                                        state::Mode::Pen => {
                                            events::mouse_pressed_pen(position, &v, canvas, button)
                                        }
                                        state::Mode::Select => events::mouse_pressed_select(
                                            position, &v, canvas, button,
                                        ),
                                        _ => false,
                                    };
                                }
                                ElementState::Released => {
                                    match mode {
                                        state::Mode::Pen => {
                                            events::mouse_released_pen(position, &v, canvas, button)
                                        }
                                        state::Mode::Select => events::mouse_released_select(
                                            position, &v, canvas, button,
                                        ),
                                        state::Mode::Zoom => events::mouse_released_zoom(
                                            position, &v, canvas, button,
                                        ),
                                        _ => false,
                                    };
                                }
                            }
                            renderer::update_viewport(canvas);
                            renderer::render_frame(canvas);
                        });
                    });
                }
                _ => (),
            },
            Event::RedrawRequested { .. } => {
                if let Err(e) = renderer.draw(&window, |canvas, coordinate_system_helper| {
                    imgui_manager.begin_frame(&winit_window);
                    frame_count += 1;

                    renderer::update_viewport(canvas);
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
