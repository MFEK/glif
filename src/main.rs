#![feature(assoc_char_funcs, panic_info_message)]

#[macro_use] extern crate lazy_static;
extern crate skia_safe;
extern crate xmltree;
extern crate gl;
extern crate clap;
#[macro_use] extern crate git_version; // for util::parse_args
#[macro_use] extern crate conrod;
extern crate glium;
use glium::glutin;

#[macro_use] mod util;
// Provides a thread-local global `state` variable
mod state;
use state::{Glyph, state};
mod renderer;
mod glifparser;
mod events;

static CLEAR_COLOR: u32 = 0xff_c4c4c4;
static SCALE_FACTOR: f32 = 0.05;
static OFFSET_FACTOR: f32 = 10.;

use std::env;
use std::fs;

use skia_safe::gpu::gl::FramebufferInfo;
use skia_safe::gpu::{BackendRenderTarget, SurfaceOrigin};
use skia_safe::{Color, ColorType, Surface, Matrix, Canvas};
use std::convert::TryInto;

use glutin::event::{Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent, ElementState};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::GlProfile;
use glutin::dpi::PhysicalPosition;

type WindowedContext = glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>;

use gl::types::*;
use std::cell::RefCell;

fn main() {
    util::set_panic_hook();

    let args = util::argparser::parse_args();
    let filename = args.filename;

    let el = EventLoop::new();
    let wb = WindowBuilder::new().with_title(format!("Qglif: {}", filename));

    let cb = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_stencil_buffer(8)
        .with_pixel_format(24, 8)
        .with_double_buffer(Some(true))
        .with_gl_profile(GlProfile::Core);

    let windowed_context = cb.build_windowed(wb, &el).unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };
    let pixel_format = windowed_context.get_pixel_format();

    debug!("Pixel format of the window's GL context: {:?}", pixel_format);

    gl::load_with(|s| windowed_context.get_proc_address(&s));

    let mut gr_context = skia_safe::gpu::Context::new_gl(None).unwrap();

    let fb_info = {
        let mut fboid: GLint = 0;
        unsafe { gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid) };

        FramebufferInfo {
            fboid: fboid.try_into().unwrap(),
            format: skia_safe::gpu::gl::Format::RGBA8.into(),
        }
    };

    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 800;

    windowed_context
        .window()
        .set_inner_size(glutin::dpi::Size::new(glutin::dpi::LogicalSize::new(
            WIDTH, HEIGHT
        )));

    let mut conrod_ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();
    let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();
    conrod_ui.draw();

    fn create_surface(
        windowed_context: &WindowedContext,
        fb_info: &FramebufferInfo,
        gr_context: &mut skia_safe::gpu::Context,
    ) -> skia_safe::Surface {
        let pixel_format = windowed_context.get_pixel_format();
        let size = windowed_context.window().inner_size();
        let backend_render_target = BackendRenderTarget::new_gl(
            (
                size.width.try_into().unwrap(),
                size.height.try_into().unwrap(),
            ),
            pixel_format.multisampling.map(|s| s.try_into().unwrap()),
            pixel_format.stencil_bits.try_into().unwrap(),
            *fb_info,
        );
        Surface::from_backend_render_target(
            gr_context,
            &backend_render_target,
            SurfaceOrigin::BottomLeft,
            ColorType::RGBA8888,
            None,
            None,
        )
        .unwrap()
    };

    let mut surface = create_surface(&windowed_context, &fb_info, &mut gr_context);

    let mut frame = 0;

    let glif = glifparser::read_ufo_glif(&fs::read_to_string(&filename).expect("Failed to read file"));
    state.with(|v|v.borrow_mut().glyph = Some(Glyph{glif, filename: filename.clone()}));
    state.with(|v| {
        v.borrow().glyph.as_ref().map(|glyph| {
            let glif = &glyph.glif;
            debug!("Loaded {:?} (U+{:04x}) from {}", glif.name, glif.unicode, state.with(|v|v.borrow().glyph.as_ref().expect("Glyph NULL!?").filename.clone()));
        });
    });

    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        #[allow(deprecated)]
        match event {
            Event::LoopDestroyed => {}
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    surface = create_surface(&windowed_context, &fb_info, &mut gr_context);
                    windowed_context.resize(physical_size);
                    state.with(|v| {
                        { v.borrow_mut().winsize = physical_size; }
                    });
                },
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
                    if kstate != ElementState::Pressed { return }
                    let canvas = surface.canvas();
                    let mut scale = state.with(|v|v.borrow().factor);
                    let mut offset = state.with(|v|v.borrow().offset);
                    match virtual_keycode {
                        // Quit
                        Some(VirtualKeyCode::Q) => {
                            *control_flow = ControlFlow::Exit;
                            return
                        },
                        // Scales
                        Some(VirtualKeyCode::Key1) => { scale = 1. },
                        Some(VirtualKeyCode::Equals) => {
                            scale += SCALE_FACTOR;
                        },
                        Some(VirtualKeyCode::Minus) => {
                            if scale >= 0.10 {
                                scale += -SCALE_FACTOR;
                            } else {
                                return
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
                        _ => { return }
                    }
                    state.with(|v|events::update_viewport(Some(offset), Some(scale), &v, canvas));
                    frame += 1;
                    windowed_context.window().request_redraw();
                    debug!("Scale factor now {}", state.with(|v|v.borrow().factor));
                },
                WindowEvent::CursorMoved{ position, .. } => {
                    let mut canvas = surface.canvas();
                    state.with(|v| {
                        let mode = v.borrow().mode;
                        
                        let should_redraw = match mode {
                            state::Mode::Select => { events::mouse_moved_select(position, &v, &mut canvas) },
                            state::Mode::Move => { events::mouse_moved_move(position, &v, &mut canvas) }
                        };

                        if should_redraw { windowed_context.window().request_redraw(); }
                    });
                },
                WindowEvent::MouseInput{
                    state: mstate, button, ..
                } => {
                    if button != MouseButton::Left { return }
                    state.with(|v| { 
                        { v.borrow_mut().mousedown = mstate == ElementState::Pressed; }
                        if v.borrow().mode == state::Mode::Select { v.borrow_mut().show_sel_box = mstate == ElementState::Pressed; }
                        match mstate {
                            ElementState::Pressed => {
                                let position = v.borrow().mousepos;
                                let mposition = PhysicalPosition::from((position.x, position.y));                                
                                v.borrow_mut().mousepos = mposition;
                                if v.borrow().show_sel_box {
                                    v.borrow_mut().corner_one = Some(mposition);
                                }
                            },
                            ElementState::Released => {
                                v.borrow_mut().show_sel_box = false;
                                windowed_context.window().request_redraw();
                            }
                        }
                    });
                },
                _ => (),
            },
            Event::RedrawRequested(_) => {
                let canvas = surface.canvas();
                canvas.clear(CLEAR_COLOR);
                state.with(|v|renderer::render_frame(frame % 360, 12, 60, canvas));
                canvas.flush();
                windowed_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}
