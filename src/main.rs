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

extern crate skulpin;

extern crate clipboard;
extern crate regex;

// Our crates
extern crate glifparser;
extern crate mfek_ipc;
extern crate xmltree;

extern crate sdl2;

use command::{Command, CommandInfo};
use imgui as imgui_rs;
use imgui_rs::{Context, DrawData, FontAtlasRef};

//use renderer::render_frame;
use sdl2::{event::{self, Event, WindowEvent}, keyboard::Mod};
use sdl2::keyboard::Keycode;
use skulpin::{CoordinateSystemHelper, LogicalSize, RendererBuilder, rafx::api::RafxExtents2D, rafx::api::RafxQueueType, skia_safe::{Matrix, RCHandle, SamplingOptions}};
pub use skulpin::skia_safe;

use skulpin::rafx::api::ash;
use ash::vk;

use crate::skia_safe::{Contains, Point};

use enum_iterator::IntoEnumIterator as _;

use std::{borrow::BorrowMut, collections::HashSet, convert::TryInto, env};
use std::time::Instant;

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
mod user_interface;
mod io;
mod ipc;
mod renderer;
mod system_fonts;
mod command;

use crate::renderer::constants::*;

fn main() {
    let window_size = (WIDTH, HEIGHT);
    let args = util::argparser::parse_args();
    let filename = filedialog::filename_or_panic(&args.filename, Some("glif"), None);
    let _glif = io::load_glif(&filename);

    // events for on_load_glif go here
    events::vws::on_load_glif();

    if mfek_ipc::module_available("MFEKmetadata".into()) == mfek_ipc::Available::Yes {
        ipc::fetch_metrics();
    }

    // SDL initialization
    let sdl_context = sdl2::init().expect("Failed to initialize sdl2");
    let video_subsystem = sdl_context
        .video()
        .expect("Failed to create sdl video subsystem");

    let logical_size = LogicalSize {
        width: WIDTH,
        height: HEIGHT,
    };

    let scale_to_fit = skulpin::skia_safe::matrix::ScaleToFit::Start;
    let visible_range = skulpin::skia_safe::Rect {
        left: 0.0,
        right: logical_size.width as f32,
        top: 0.0,
        bottom: logical_size.height as f32,
    };

    let window = video_subsystem
        .window("MFEKglif", logical_size.width, logical_size.height)
        .position_centered()
        .allow_highdpi()
        .vulkan()
        .resizable()
        .build()
        .expect("Failed to create window");

    
    // Skulpin initialization 
    let (window_width, window_height) = window.vulkan_drawable_size();

    let extents = RafxExtents2D {
        width: window_width,
        height: window_height,
    };

    let renderer = RendererBuilder::new()
        .coordinate_system(skulpin::CoordinateSystem::VisibleRange(
            visible_range,
            scale_to_fit,
        ))
        .build(&window, extents);

    // Check if there were error setting up vulkan
    if let Err(e) = renderer {
        println!("Error during renderer construction: {:?}", e);
        return;
    }

    let mut renderer = renderer.unwrap();

    // set up imgui
    let mut imgui = setup_imgui();
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

        // event handling
        for event in event_pump.poll_iter() {
            imgui_sdl2.handle_event(&mut imgui, &event);
            if imgui_sdl2.ignore_event(&event) { continue; };

            // we're gonna handle some of these events before handling commands so that we don't have the logic for this stuff
            // intertwined in command handling
            match event {
                Event::Quit { .. } => break 'main_loop,
                Event::KeyDown { keycode: Some(Keycode::Q), keymod: km,  .. } => {
                    if km.contains(Mod::LSHIFTMOD | Mod::RSHIFTMOD){
                        break 'main_loop;
                    }
                }
                Event::KeyDown { keycode: Some(Keycode::S), keymod: km, .. } => {
                    if km.contains(Mod::LSHIFTMOD | Mod::RSHIFTMOD){
                        STATE.with(|v| {
                            io::save_glif(v);
                        });
                        continue;
                    }
                }
                Event::KeyDown { keycode: Some(Keycode::E), keymod: km, .. } => {
                    if km.contains(Mod::LSHIFTMOD | Mod::RSHIFTMOD){
                        STATE.with(|v| {
                            io::export_glif(v);
                        });
                        continue;
                    }
                }
                _ => {}
            }

            match event {
                Event::KeyDown { keycode, keymod, ..} => {
                    STATE.with(|v| {
                        let mode = v.borrow().mode;
                        let mut newmode = mode;
                        let mut scale = v.borrow().factor;
                        let mut offset = v.borrow().offset;

                        let command: Option<CommandInfo> = match keycode {
                            Some(keycode) => {
                                command::keycode_to_command(&keycode, &keys_down)
                            }

                            _ => None
                        };

                        if let Some(command_info) = command {
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
                                    trigger_toggle_on!(v, point_labels, PointLabels, command_info.command_mod.shift);
                                }
                                Command::TogglePreviewMode => {
                                    trigger_toggle_on!(v, preview_mode, PreviewMode, !command_info.command_mod.shift);
                                }

                                _ => { unreachable!("The remaining Command enums should never be returned.")}
                            }
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
                },

                Event::MouseMotion { x, y, .. } => {
                    let position = (x as f64, y as f64);
                    STATE.with(|v| {
                        let mode = v.borrow().mode;

                        match mode {
                            #[rustfmt::skip]
                            state::Mode::Pan => events::pan::mouse_moved(position, &v),
                            state::Mode::Pen => events::pen::mouse_moved(position, &v),
                            state::Mode::Select => {    events::select::mouse_moved(position, &v);
                                                        events::vws::update_previews(position, &v)},
                            state::Mode::VWS => events::vws::mouse_moved(position, &v),
                            state::Mode::Zoom => events::zoom::mouse_moved(position, &v),
                            _ => false,
                        };
                    });
                },

                Event::MouseButtonDown { mouse_btn, .. } => {
                    STATE.with(|v| {
                        let keymod = command::key_down_to_mod(&keys_down);
                        let meta = events::MouseMeta{ button: mouse_btn, modifiers: keymod };
    
                        let mode = v.borrow().mode;
                        let position = v.borrow().mousepos;
                        v.borrow_mut().mousedown = true;

                        match mode {
                            state::Mode::Select => {
                                events::select::mouse_button(position, &v, meta)
                            }
                            state::Mode::VWS => {
                                events::vws::mouse_button(position, &v, meta)
                            }
                            _ => false,
                        };

                        match mode {
                            state::Mode::Pen => {
                                events::pen::mouse_pressed(position, &v, meta)
                            }
                            state::Mode::Select => {
                                events::select::mouse_pressed(position, &v, meta)
                            }
                            state::Mode::VWS => {
                                events::vws::mouse_pressed(position, &v, meta)
                            }
                            _ => false,
                        };
                    });
                },


                Event::MouseButtonUp { mouse_btn , .. } => {
                    STATE.with(|v| {
                        let keymod = command::key_down_to_mod(&keys_down);
                        let meta = events::MouseMeta{ button: mouse_btn, modifiers: keymod };
    
                        let mode = v.borrow().mode;
                        let position = v.borrow().mousepos;
                        v.borrow_mut().mousedown = false;

                        match mode {
                            state::Mode::Pen => {
                                events::pen::mouse_released(position, &v, meta)
                            }
                            state::Mode::Select => {
                                events::select::mouse_released(position, &v, meta)
                            }
                            state::Mode::Zoom => {
                                events::zoom::mouse_released(position, &v, meta);
                                events::center_cursor(&sdl_context, &window);
                                true
                            }
                            state::Mode::VWS => {
                                events::vws::mouse_released(position, &v, meta)
                            }
                            _ => false,
                        };
                    });
                },

                Event::Window {win_event, .. } => {
                    match win_event {
                        WindowEvent::Resized(x, y) => {
                            STATE.with(|v| {
                                v.borrow_mut().winsize = (x as u32, y as u32);
                            });
                        }

                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // build and render imgui
        imgui_sdl2.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());
        let mut ui = imgui.frame();
        crate::user_interface::build_imgui_ui(&mut ui);

        imgui_sdl2.prepare_render(&ui, &window);           
        let dd = ui.render();


        // draw glyph preview and imgui with skia
        let (window_width, window_height) = window.vulkan_drawable_size();
        let extents = RafxExtents2D {
            width: window_width,
            height: window_height,
        };


        renderer
            .draw(extents, 1.0, |canvas, coordinate_system_helper| {
                renderer::render_frame(canvas);
                imgui_renderer.render_imgui(canvas, dd);
            })
            .unwrap();

    }

}

fn setup_imgui() -> Context 
{
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
            data: &system_fonts::SYSTEMSANS.data,
            size_pixels: font_size,
            config: Some(imgui::FontConfig {
                oversample_h: 3,
                oversample_v: 3,
                ..Default::default()
            }),
        },
        imgui_rs::FontSource::TtfData {
            data: include_bytes!("../resources/fonts/icons.ttf"),
            size_pixels: icon_font_size,
            config: Some(imgui_rs::FontConfig {
                glyph_ranges: imgui_rs::FontGlyphRanges::from_slice(&[
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

struct Renderer {
    // this holds the skia formatted font atlas
    skfont_paint: skia_safe::Paint,
}

impl Renderer {
    fn build_paint(atlas: &mut imgui::FontAtlasRefMut, font_paint: &mut skia_safe::Paint)
    {
        let imfont_texture = atlas.build_alpha8_texture();
        let dimensions = skia_safe::ISize::new(imfont_texture.width as i32, imfont_texture.height as i32);
        let imgfont_a8 = skia_safe::ImageInfo::new_a8(dimensions);
        
        let pixels = unsafe {
            skia_safe::Data::new_bytes(imfont_texture.data)
        };

        let pixmap = skia_safe::Pixmap::new(&imgfont_a8, imfont_texture.data, imgfont_a8.min_row_bytes());
        let font_image = skia_safe::Image::from_raster_data(&imgfont_a8, pixels, pixmap.row_bytes());

        let local_matrix = skia_safe::Matrix::scale((1.0 / imfont_texture.width as f32, 1.0 / imfont_texture.height as f32));
        let sampling_options = skia_safe::SamplingOptions::new(skia_safe::FilterMode::Nearest, skia_safe::MipmapMode::None);
        let tile_mode = skia_safe::TileMode::Repeat;

        let font_shader = font_image.unwrap().to_shader((tile_mode, tile_mode), sampling_options, &local_matrix);

        font_paint.set_shader(font_shader);
        font_paint.set_color(skia_safe::Color::WHITE);
    }

    pub fn new(im_context: &mut Context) -> Self
    {
        let mut font_paint = skia_safe::Paint::default();
        Self::build_paint(&mut im_context.fonts(), &mut font_paint);
    
        Renderer {
            skfont_paint: font_paint,
        }
    }

    pub fn render_imgui(&self, canvas: &mut skia_safe::Canvas, data: &DrawData, )
    {
        canvas.save();
        let mut matrix = Matrix::new_identity();
        matrix.set_scale((1., 1.), None);
    
        canvas.set_matrix(&matrix.into());
        for draw_list in data.draw_lists() {
            let mut idx: Vec<u16> = Vec::new();
            let mut pos: Vec<skia_safe::Point> = Vec::new();
            let mut uv: Vec<skia_safe::Point> = Vec::new();
            let mut color: Vec<skia_safe::Color> = Vec::new();

            // we've got to translate the vertex buffer from imgui into Skia friendly types
            // thankfully skia_safe gives us a constructor for Color so we don't have to swizzle the colors as Skia expects BGR order
            for vertex in draw_list.vtx_buffer() {
                pos.push(skia_safe::Point {
                    x: vertex.pos[0],
                    y: vertex.pos[1]
                });

                uv.push(skia_safe::Point {
                    x: vertex.uv[0],
                    y: vertex.uv[1]
                });

                color.push(skia_safe::Color::from_argb(
                    vertex.col[3],
                    vertex.col[0],
                    vertex.col[1],
                    vertex.col[2],
                ));
            }
            
            // we build our index buffer
            for index in draw_list.idx_buffer() {
                idx.push(*index);
            }

            // so now we've got to loop through imgui's cmd buffer and draw everything with canvas.draw_vertices
            for cmd in draw_list.commands() {
                let mut arc = skia_safe::AutoCanvasRestore::guard(canvas, true);
                match cmd {
                    imgui::DrawCmd::RawCallback {
                        callback,
                        raw_cmd
                    } => {
                        todo!("Raw callbacks unimplemented!")
                    }
                    imgui::DrawCmd::ResetRenderState => {
                        todo!("Reset render state unimplemented!")
                    }
                    imgui::DrawCmd::Elements {
                        count,
                        cmd_params,
                    } => {
                        //TODO: Handle images that aren't our font atlas
                        let id_index = cmd_params.texture_id;

                        let clip_rect = cmd_params.clip_rect;
                        let skclip_rect = skia_safe::Rect::new(clip_rect[0], clip_rect[1], clip_rect[2], clip_rect[3]);

                        let vertex_mode = skia_safe::vertices::VertexMode::Triangles;
                        let idx_offset = cmd_params.idx_offset;
                        let idx_slice = Some(&idx[idx_offset .. idx_offset + count]);

                        arc.clip_rect(skclip_rect, skia_safe::ClipOp::default(), true);
                        let vertices = skia_safe::Vertices::new_copy(vertex_mode, &pos, &uv, &color, idx_slice);
                        arc.draw_vertices(&vertices, skia_safe::BlendMode::Modulate, &self.skfont_paint);
                    }
                    _ => {}
                }
            }
        }
    }
}