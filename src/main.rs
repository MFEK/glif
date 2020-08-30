#![feature(assoc_char_funcs, panic_info_message)]

#[macro_use] extern crate lazy_static;
extern crate thiserror;
#[macro_use] extern crate glium;
extern crate skia_safe;
extern crate xmltree;
extern crate gl;
extern crate clap;
#[macro_use] extern crate git_version; // for util::parse_args
extern crate skia_safe as skia;

use glium::glutin;
use glutin::dpi::PhysicalPosition;
use glutin::event::{ElementState, Event, MouseButton, WindowEvent, KeyboardInput, VirtualKeyCode};
use glutin::event_loop::{ControlFlow, EventLoop};
use glium::{GlObject, Surface};

use std::time::Instant;

#[macro_use] extern crate imgui; // for the macros, can't use one in imgui_glium_renderer
#[macro_use] extern crate imgui_glium_renderer;
extern crate imgui_winit_support;
use imgui_winit_support::WinitPlatform;
use imgui_glium_renderer::Renderer as ImguiRenderer;
use imgui::{Context as ImguiContext};

mod reclutch_skia;
#[macro_use] mod util;
// Provides a thread-local global `state` variable
mod state;
use state::{Glyph, state};
mod renderer;
mod glifparser;
mod events;

use renderer::constants::*;

#[derive(Copy, Clone)]
struct TextureVertex {
    position: [f32; 3],
    tex_coord: [f32; 2],
}

implement_vertex!(TextureVertex, position, tex_coord);

const fn texture_vertex(pos: [i8; 2], tex: [i8; 2]) -> TextureVertex {
    TextureVertex {
        position: [pos[0] as _, pos[1] as _, 0.0],
        tex_coord: [tex[0] as _, tex[1] as _],
    }
}

const QUAD_VERTICES: [TextureVertex; 4] = [
    texture_vertex([-1, -1], [0, 0]),
    texture_vertex([-1, 1], [0, 1]),
    texture_vertex([1, 1], [1, 1]),
    texture_vertex([1, -1], [1, 0]),
];

const QUAD_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

fn run_ui(ui: &mut imgui::Ui) {
    imgui::Window::new(im_str!("Hello world"))
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

const HEIGHT: u32 = 800;
const WIDTH: u32 = HEIGHT;

use std::fs;
fn main() {
    let window_size = (WIDTH, HEIGHT);

    let args = util::argparser::parse_args();
    let filename = args.filename;
    let glif = glifparser::read_ufo_glif(&fs::read_to_string(&filename).expect("Failed to read file"));
    state.with(|v|v.borrow_mut().glyph = Some(Glyph{glif, filename: filename.clone()}));
    state.with(|v| {
        v.borrow().glyph.as_ref().map(|glyph| {
            let glif = &glyph.glif;
            debug!("Loaded {:?} (U+{:04x}) from {}", glif.name, glif.unicode, state.with(|v|v.borrow().glyph.as_ref().expect("Glyph NULL!?").filename.clone()));
        });
    });

    let event_loop = EventLoop::new();

    let wb = glutin::window::WindowBuilder::new()
        .with_title(format!("Qglif: {}", filename))
        .with_inner_size(glutin::dpi::PhysicalSize::new(window_size.0 as f64, window_size.1 as f64))
        .with_resizable(false);

    let cb = glutin::ContextBuilder::new().with_vsync(true).with_srgb(true);

    let gl_display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let quad_vertex_buffer = glium::VertexBuffer::new(&gl_display, &QUAD_VERTICES).unwrap();
    let quad_indices = glium::IndexBuffer::new(
        &gl_display,
        glium::index::PrimitiveType::TrianglesList,
        &QUAD_INDICES,
    )
    .unwrap();

    let quad_vertex_shader_src = r#"
        #version 140

        in vec3 position;
        in vec2 tex_coord;

        out vec2 frag_tex_coord;

        void main() {
            frag_tex_coord = tex_coord;
            gl_Position = vec4(position, 1.0);
        }
    "#;

    let quad_fragment_shader_src = r#"
        #version 150

        in vec2 frag_tex_coord;
        out vec4 color;

        uniform sampler2D tex;

        void main() {
            color = texture(tex, frag_tex_coord);
        }
    "#;

    let quad_program = glium::Program::from_source(
        &gl_display,
        quad_vertex_shader_src,
        quad_fragment_shader_src,
        None,
    )
    .unwrap();

    let out_texture = glium::texture::SrgbTexture2d::empty_with_format(
        &gl_display,
        glium::texture::SrgbFormat::U8U8U8U8,
        glium::texture::MipmapsOption::NoMipmap,
        window_size.0,
        window_size.1,
    )
    .unwrap();
    let out_texture_depth =
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


    let mut display =
        reclutch_skia::SkiaGraphicsDisplay::new_gl_texture(&reclutch_skia::SkiaOpenGlTexture {
            size: (window_size.0 as _, window_size.1 as _),
            texture_id: out_texture.get_id(),
            mip_mapped: false,
        })
        .unwrap();

    let mut last_frame = Instant::now();

    let mut imgui = ImguiContext::create();
    let mut platform = WinitPlatform::init(&mut imgui);
    imgui.set_ini_filename(None);
    imgui.io_mut().display_size = [window_size.0 as f32, window_size.1 as f32];
    let mut renderer = ImguiRenderer::init(&mut imgui, &gl_display).expect("Failed to initialize renderer");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(
            std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667),
        );

        platform.handle_event(imgui.io_mut(), &gl_display.gl_window().window(), &event);

        #[allow(deprecated)]
        match event {
            Event::LoopDestroyed => {}
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    /*surface = create_surface(&display, &fb_info, &mut gr_context);
                    gl_display.gl_window().resize(physical_size);
                    state.with(|v| {
                        { v.borrow_mut().winsize = physical_size; }
                    });*/
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
                    let canvas = display.surface.canvas();
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
                    //frame += 1;
                    gl_display.gl_window().window().request_redraw();
                    debug!("Scale factor now {}", state.with(|v|v.borrow().factor));
                },
                WindowEvent::CursorMoved{ position, .. } => {
                    let mut canvas = display.surface.canvas();
                    state.with(|v| {
                        let mode = v.borrow().mode;
                        
                        let should_redraw = match mode {
                            state::Mode::Select => { events::mouse_moved_select(position, &v, &mut canvas) },
                            state::Mode::Move => { events::mouse_moved_move(position, &v, &mut canvas) }
                        };

                        if should_redraw { gl_display.gl_window().window().request_redraw(); }
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
                                gl_display.gl_window().window().request_redraw();
                            }
                        }
                    });
                },
                _ => (),
            },
            Event::RedrawRequested { .. } => {
                let mut out_texture_fb = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(
                    &gl_display,
                    &out_texture,
                    &out_texture_depth,
                )
                .unwrap();

                let mut frame_target = gl_display.draw();
                let target = &mut out_texture_fb;

                target.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);

                skia_context =
                    Some(unsafe { skia_context.take().unwrap().make_current().unwrap() });

                render_skia(&mut display);
                render_imgui_frame(target, &mut imgui, &mut last_frame, &mut renderer);
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
            },
            Event::MainEventsCleared => {
                gl_display.gl_window().window().request_redraw();
            },
            Event::WindowEvent { event: WindowEvent::KeyboardInput { input: KeyboardInput { virtual_keycode: Some(VirtualKeyCode::Escape), .. }, .. }, ..} | Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            },
            _ => return,
        }
    });
}

fn render_imgui_frame(target: &mut glium::framebuffer::SimpleFrameBuffer, imgui: &mut imgui::Context, last_frame: &mut Instant, renderer: &mut ImguiRenderer) {
    let io = imgui.io_mut();

    *last_frame = io.update_delta_time(*last_frame);
    let mut ui = imgui.frame();
    run_ui(&mut ui);

    let draw_data = ui.render();
    renderer.render(target, draw_data).expect("Rendering failed");
}

fn render_skia(display: &mut reclutch_skia::SkiaGraphicsDisplay) {
    let mut surface = &mut display.surface;
    let canvas = surface.canvas();
    let count = canvas.save();
    let center = (HEIGHT as f32 / 4., WIDTH as f32 / 4.);
    state.with(|v|renderer::render_frame(0, 12, 60, canvas));
    //canvas.restore_to_count(count);
    display.surface.flush_and_submit();
}
