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
use reclutch::skia::{Contains, Rect, Point};

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
    util::set_panic_hook();

    let window_size = (WIDTH, HEIGHT);
    state.with(|v| {
        v.borrow_mut().winsize = window_size.into();
    });

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

    state.with(|v| {
        let icons = state::Icons::from_display(&gl_display, &mut renderer);
        v.borrow_mut().icons = Some(icons);
    });

    let mut should_redraw_skia = true;
    let mut frame = 0;
    let mut was_resized = false;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        platform.handle_event(imgui.io_mut(), &gl_display.gl_window().window(), &event);

        // These handle the Skia events. ImGui "events" don't really exist, see
        // src/opengl/imgui.rs:fn build_imgui_ui.
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
                        display.perform_draw_closure(|canvas, _| {
                            events::update_viewport(None, None, &v, canvas)
                        });
                    });

                    was_resized = true;
                }
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
                    state.with(|v| {
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
                            Some(VirtualKeyCode::V) => {
                                v.borrow_mut().mode = state::Mode::Select;
                            }
                            Some(VirtualKeyCode::A) => {
                                v.borrow_mut().mode = state::Mode::Pan;
                            }
                            Some(VirtualKeyCode::Z) => {
                                v.borrow_mut().mode = state::Mode::Zoom;
                            }
                            _ => {},
                        }
                        display.perform_draw_closure(|canvas, _| {
                            events::update_viewport(Some(offset), Some(scale), &v, canvas)
                        });

                        should_redraw_skia = true;
                        debug!("Scale factor now {}; offset {:?}; mode {:?}", v.borrow().factor, v.borrow().offset, v.borrow().mode);
                    });
                }
                WindowEvent::CursorMoved { position, .. } => {
                    display.perform_draw_closure(|canvas, _| {
                        state.with(|v| {
                            let mode = v.borrow().mode;

                            should_redraw_skia = match mode {
                                #[rustfmt::skip]
                                state::Mode::Select => events::mouse_moved_select(position, &v, canvas),
                                state::Mode::Pan => events::mouse_moved_move(position, &v, canvas),
                                state::Mode::Zoom => events::mouse_moved_zoom(position, &v, canvas),
                            };
                        });
                    });
                }
                WindowEvent::MouseInput {
                    state: mstate,
                    button,
                    ..
                } => {
                    // Ignore events if we are clicking on Dear ImGui toolbox.
                    state.with(|v| {
                        let toolbox_rect = opengl::imgui::toolbox_rect();
                        let absolute_position = v.borrow().absolute_mousepos;
                        if toolbox_rect.contains(Point::from((absolute_position.x as f32, absolute_position.y as f32))) {
                            return
                        }

                        display.perform_draw_closure(|canvas, _| {
                            let mode = v.borrow().mode;
                            let position = v.borrow().mousepos;
                            v.borrow_mut().mousedown = mstate == ElementState::Pressed;

                            should_redraw_skia = match mode {
                                state::Mode::Select => {
                                    events::mouse_button_select(position, &v, canvas, button)
                                }
                                _ => false,
                            };

                            match mstate {
                                ElementState::Pressed => {
                                    should_redraw_skia = match mode {
                                        state::Mode::Select => events::mouse_pressed_select(
                                            position, &v, canvas, button,
                                        ),
                                        _ => false,
                                    };
                                }
                                ElementState::Released => {
                                    should_redraw_skia = match mode {
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
                        });
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

                skia_context =
                    Some(unsafe { skia_context.take().unwrap().make_current().unwrap() });

                if !should_redraw_skia && was_resized {
                    should_redraw_skia = true;
                    was_resized = false;
                }

                // Yes, 10 is a magic number. 👻
                if frame < 10 || should_redraw_skia {
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
            _ => return,
        }
        frame += 1;
    });
}
