#![feature(assoc_char_funcs, panic_info_message)]
//! Qglif - A cross-platform .glif renderer and editor.
//! Main author is Fredrick Brennan (@ctrlcctrlv); see AUTHORS.
//! (c) 2020. Apache 2.0 licensed.

// Cargo.toml comments say what crates are used for what.
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate glium;
extern crate clap;
extern crate gl;
extern crate xmltree;
#[macro_use]
extern crate git_version; // for util::parse_args
extern crate nsvg;
extern crate reclutch;

use glium::glutin;
use glium::{GlObject, Surface};
use glutin::dpi::PhysicalPosition;
use glutin::event::{ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};

use reclutch::display::GraphicsDisplay;

use std::time::Instant;

#[macro_use]
extern crate imgui; // for the macros, can't use one in imgui_glium_renderer
#[macro_use]
extern crate imgui_glium_renderer;
extern crate imgui_winit_support;
use imgui::Context as ImguiContext;
use imgui_glium_renderer::Renderer as ImguiRenderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

#[macro_use]
mod util;
// Provides a thread-local global `state` variable
mod state;
use state::{state, Glyph};
mod events;
mod glifparser;
mod opengl;
mod renderer;

use renderer::constants::*;

use std::fs;
fn main() {
    let window_size = (WIDTH, HEIGHT);

    let args = util::argparser::parse_args();
    let filename = args.filename;
    let glif =
        glifparser::read_ufo_glif(&fs::read_to_string(&filename).expect("Failed to read file"));
    state.with(|v| {
        v.borrow_mut().glyph = Some(Glyph {
            glif,
            filename: filename.clone(),
        })
    });
    state.with(|v| {
        v.borrow().glyph.as_ref().map(|glyph| {
            let glif = &glyph.glif;
            debug!(
                "Loaded {:?} (U+{:04x}) from {}",
                glif.name,
                glif.unicode,
                state.with(|v| v
                    .borrow()
                    .glyph
                    .as_ref()
                    .expect("Glyph NULL!?")
                    .filename
                    .clone())
            );
        });
    });

    let event_loop = EventLoop::new();

    let wb = glutin::window::WindowBuilder::new()
        .with_title(format!("Qglif: {}", filename))
        .with_inner_size(glutin::dpi::PhysicalSize::new(
            window_size.0 as f64,
            window_size.1 as f64,
        ))
        .with_resizable(true);

    let cb = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_srgb(true);

    let gl_display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let quad_vertex_buffer = opengl::quad_vertex_buffer(&gl_display);
    let quad_indices = opengl::quad_indices(&gl_display);
    let quad_program = opengl::quad_program(&gl_display);
    let mut out_texture = opengl::create_texture(&gl_display, window_size);

    let mut out_texture_depth =
        glium::texture::DepthTexture2d::empty(&gl_display, window_size.0, window_size.1).unwrap();

    let mut skia_context = Some(unsafe {
        glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 3)))
            .with_shared_lists(&gl_display.gl_window())
            .with_srgb(true)
            .build_headless(
                &event_loop,
                glutin::dpi::PhysicalSize::new(window_size.0 as _, window_size.1 as _),
            )
            .unwrap()
            .make_current()
            .unwrap()
    });

    let mut display = opengl::skia::make_skia_display(&out_texture, window_size);

    let mut last_frame = Instant::now();

    let mut imgui = ImguiContext::create();
    let mut platform = WinitPlatform::init(&mut imgui);
    platform.attach_window(
        imgui.io_mut(),
        &gl_display.gl_window().window(),
        HiDpiMode::Default,
    );
    debug!("DPI is {}", gl_display.gl_window().window().scale_factor());

    imgui.set_ini_filename(None);

    state.with(|v| {
        v.borrow_mut().dpi = gl_display.gl_window().window().scale_factor();
    });

    opengl::imgui::set_imgui_fonts(&mut imgui);
    opengl::imgui::set_imgui_dpi(&mut imgui, window_size);

    let mut renderer =
        ImguiRenderer::init(&mut imgui, &gl_display).expect("Failed to initialize renderer");

    let mut should_redraw_skia = true;
    let mut frame = 0;
    let mut was_resized = false;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        platform.handle_event(imgui.io_mut(), &gl_display.gl_window().window(), &event);

        #[allow(deprecated)]
        match event {
            Event::LoopDestroyed => {}
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    out_texture = opengl::create_texture(
                        &gl_display,
                        (physical_size.width, physical_size.height),
                    );
                    out_texture_depth = glium::texture::DepthTexture2d::empty(
                        &gl_display,
                        physical_size.width,
                        physical_size.height,
                    )
                    .unwrap();

                    gl_display.gl_window().resize(physical_size);
                    opengl::imgui::set_imgui_dpi(
                        &mut imgui,
                        (physical_size.width, physical_size.height),
                    );

                    skia_context =
                        Some(unsafe { skia_context.take().unwrap().make_current().unwrap() });
                    display = opengl::skia::make_skia_display(
                        &out_texture,
                        (physical_size.width, physical_size.height),
                    );
                    should_redraw_skia = true;

                    state.with(|v| {
                        v.borrow_mut().winsize = physical_size;
                    });

                    was_resized = true;
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
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
                    let mut scale = state.with(|v| v.borrow().factor);
                    let mut offset = state.with(|v| v.borrow().offset);
                    match virtual_keycode {
                        // Quit
                        Some(VirtualKeyCode::Q) => {
                            *control_flow = ControlFlow::Exit;
                            return;
                        }
                        // Scales
                        Some(VirtualKeyCode::Key1) => scale = 1.,
                        Some(VirtualKeyCode::Equals) => {
                            scale += SCALE_FACTOR;
                        }
                        Some(VirtualKeyCode::Minus) => {
                            if scale >= 0.10 {
                                scale += -SCALE_FACTOR;
                            } else {
                                return;
                            }
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
                        _ => return,
                    }
                    display.perform_draw_closure(|canvas, _| {
                        state.with(|v| {
                            events::update_viewport(Some(offset), Some(scale), &v, canvas)
                        });
                    });
                    //frame += 1;
                    gl_display.gl_window().window().request_redraw();
                    should_redraw_skia = true;
                    debug!("Scale factor now {}", state.with(|v| v.borrow().factor));
                }
                WindowEvent::CursorMoved { position, .. } => {
                    display.perform_draw_closure(|canvas, _| {
                        state.with(|v| {
                            let mode = v.borrow().mode;

                            should_redraw_skia = match mode {
                                state::Mode::Select => {
                                    events::mouse_moved_select(position, &v, canvas)
                                }
                                state::Mode::Move => events::mouse_moved_move(position, &v, canvas),
                            };
                        });
                    });
                }
                WindowEvent::MouseInput {
                    state: mstate,
                    button,
                    ..
                } => {
                    if button != MouseButton::Left {
                        return;
                    }
                    state.with(|v| {
                        {
                            v.borrow_mut().mousedown = mstate == ElementState::Pressed;
                        }
                        if v.borrow().mode == state::Mode::Select {
                            v.borrow_mut().show_sel_box = mstate == ElementState::Pressed;
                        }
                        match mstate {
                            ElementState::Pressed => {
                                let position = v.borrow().mousepos;
                                let mposition = PhysicalPosition::from((position.x, position.y));
                                v.borrow_mut().mousepos = mposition;
                                if v.borrow().show_sel_box {
                                    v.borrow_mut().corner_one = Some(mposition);
                                }
                            }
                            ElementState::Released => {
                                v.borrow_mut().show_sel_box = false;
                                gl_display.gl_window().window().request_redraw();
                                should_redraw_skia = true;
                            }
                        }
                    });
                }
                _ => (),
            },
            Event::RedrawRequested { .. } => {
                let mut out_texture_fb = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(
                    &gl_display,
                    &out_texture,
                    &out_texture_depth,
                )
                .unwrap();

                let target = &mut out_texture_fb;

                //target.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);

                skia_context =
                    Some(unsafe { skia_context.take().unwrap().make_current().unwrap() });

                if !should_redraw_skia && was_resized {
                    should_redraw_skia = true;
                    was_resized = false;
                }

                // Yes, 4 is a magic number. 👻
                if frame < 4 || should_redraw_skia {
                    display.perform_draw_closure(|canvas, _| {
                        opengl::skia::redraw_skia(canvas, &mut should_redraw_skia);
                    });
                }
                display.present(None).unwrap();

                opengl::imgui::render_imgui_frame(
                    target,
                    &mut imgui,
                    &mut last_frame,
                    &mut renderer,
                );
                let mut frame_target = gl_display.draw();
                frame_target
                    .draw(
                        &quad_vertex_buffer,
                        &quad_indices,
                        &quad_program,
                        &uniform! { tex: &out_texture },
                        &Default::default(),
                    )
                    .unwrap();

                frame_target.finish().unwrap();
            }
            Event::MainEventsCleared => {
                gl_display.gl_window().window().request_redraw();
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    },
                ..
            }
            | Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => return,
        }
        frame += 1;
    });
}
