//! MFEKglif - A cross-platform .glif renderer and editor.
//! Main author is Fredrick Brennan (@ctrlcctrlv); see AUTHORS.
//! (c) 2020. Apache 2.0 licensed.
#![allow(non_snake_case)] // for our name MFEKglif
#![feature(
    panic_info_message,
    stmt_expr_attributes,
    cell_leak,
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

use tools::{EditorEvent, MouseEventType, ToolEnum};
use editor::Editor;
use editor::MouseInfo;
//use renderer::render_frame;
use sdl2::keyboard::Keycode;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Mod,
    video::Window,
    Sdl,
};
pub use skulpin::skia_safe;
use skulpin::{rafx::api::RafxError, rafx::api::RafxExtents2D, LogicalSize, RendererBuilder};

use imgui_skia_renderer::Renderer;

use enum_iterator::IntoEnumIterator as _;

use std::{collections::HashSet, rc::Rc};

// Provides thread-local global variables.
pub mod editor;
pub use crate::editor::{HandleStyle, PointLabels, PreviewMode}; // enums
pub use crate::editor::{CONSOLE}; // globals

mod filedialog;
#[macro_use]
pub mod util;
#[macro_use]
mod tools;
mod command;
mod io;
mod ipc;
mod renderer;
pub mod settings;
mod system_fonts;
mod user_interface;
mod contour_operations;

use crate::renderer::constants::*;

struct WindowSettings {
    filename: String,
}

fn main() {
    env_logger::init();
    util::set_panic_hook();

    let args = util::argparser::parse_args();
    let filename = filedialog::filename_or_panic(&args.filename, Some("glif"), None);

    let mut editor = Editor::new();
    // Makes glyph available to on_load_glif events
    let _glif = io::load_glif(&mut editor, &filename);

    if mfek_ipc::module_available("MFEKmetadata".into()) == mfek_ipc::Available::Yes {
        ipc::fetch_metrics(&mut editor);
    }

    let (sdl_context, sdl_window): (Sdl, Window) = initialize_sdl(&mut editor, &WindowSettings {
        filename: filename.to_str().unwrap().to_string(),
    });
    editor.sdl_context = Some(sdl_context);
    editor.sdl_window = Some(sdl_window);

    // Skulpin initialization TODO: proper error handling
    let mut renderer = initialize_skulpin_renderer(&editor.sdl_window.as_ref().unwrap()).unwrap();

    // set up imgui
    let mut imgui = user_interface::setup_imgui();
    let mut imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &editor.sdl_window.as_ref().unwrap());
    let imgui_renderer = Renderer::new(&mut imgui);

    let mut event_pump = editor.sdl_context
        .as_ref()
        .unwrap()
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

        let keymod = command::key_down_to_mod(&keys_down);

        // sdl event handling
        for event in event_pump.poll_iter() {
            imgui_sdl2.handle_event(&mut imgui, &event);
            if imgui_sdl2.ignore_event(&event) {
                continue;
            };

            match &event {
                Event::Quit { .. } => break 'main_loop,
                _ => {}
            }

            if !editor.prompts.is_empty() { continue; }
            // we're gonna handle some of these events before handling commands so that we don't have the logic for this stuff
            // intertwined in command handling
            match &event {
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
                    if km.contains(Mod::LCTRLMOD) || km.contains(Mod::RCTRLMOD) {
                        io::save_glif(&mut editor);
                        continue;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::E),
                    keymod: km,
                    ..
                } => {
                    if km.contains(Mod::LCTRLMOD) || km.contains(Mod::RCTRLMOD) {
                        io::export_glif(&editor);
                        continue;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Z),
                    keymod: km,
                    ..
                } => {
                    if km.contains(Mod::LCTRLMOD) || km.contains(Mod::RCTRLMOD) {
                        editor.undo();
                        continue;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Y),
                    keymod: km,
                    ..
                } => {
                    if km.contains(Mod::LCTRLMOD) || km.contains(Mod::RCTRLMOD) {
                        editor.redo();
                        continue;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    keymod: km,
                    ..
                } => {
                    if km.contains(Mod::LCTRLMOD) || km.contains(Mod::RCTRLMOD) {
                        editor.copy_selection();
                        continue;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::V),
                    keymod: km,
                    ..
                } => {
                    if km.contains(Mod::LCTRLMOD) || km.contains(Mod::RCTRLMOD) {
                        editor.paste_selection(editor.mouse_info.position);
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

                    tools::console::set_state(&mut editor, keycode, keymod);
                    if CONSOLE.with(|c| c.borrow_mut().active) {
                        continue;
                    }

                    // check if we've got a command
                    let command_info: CommandInfo = match command::keycode_to_command(&keycode, &keys_down) {
                        Some(command) => command,
                        None => continue
                    };

                    let mut delete_after = false;
                    editor.dispatch_editor_event(EditorEvent::ToolCommand {
                        command: command_info.command,
                        command_mod: command_info.command_mod,
                        stop_after: &mut delete_after,
                    });

                    match command_info.command {
                        Command::ResetScale => {
                            editor.update_viewport(None, Some(1.));
                        }
                        Command::ZoomIn => {
                            let scale = tools::zoom_in_factor(editor.viewport.factor, &mut editor);
                            editor.update_viewport(None, Some(scale));
                        }
                        Command::ZoomOut => {
                            let scale = tools::zoom_out_factor(editor.viewport.factor, &mut editor);
                            editor.update_viewport(None, Some(scale));
                        }
                        Command::NudgeUp => {
                            editor.update_viewport(Some((0., OFFSET_FACTOR)), None);
                        }
                        Command::NudgeDown => {
                            editor.update_viewport(Some((0., -OFFSET_FACTOR)), None);
                        }
                        Command::NudgeLeft => {
                            editor.update_viewport(Some((OFFSET_FACTOR, 0.)), None);
                        }
                        Command::NudgeRight => {
                            editor.update_viewport(Some((-OFFSET_FACTOR, 0.)), None);
                        }
                        Command::ToolPan => {
                            editor.set_tool(ToolEnum::Pan);
                        }
                        Command::ToolPen => {
                            editor.set_tool(ToolEnum::Pen);
                        }
                        Command::ToolSelect => {
                            editor.set_tool(ToolEnum::Select);
                        }
                        Command::ToolZoom => {
                            editor.set_tool(ToolEnum::Zoom);
                        }
                        Command::ToolVWS => {
                            editor.set_tool(ToolEnum::VWS);
                        }
                        Command::ToolMeasure => {
                            editor.set_tool(ToolEnum::Measure);
                        }
                        Command::ToolAnchors => {
                            editor.set_tool(ToolEnum::Anchors);
                        }
                        Command::ToolShapes => {
                            editor.set_tool(ToolEnum::Shapes);
                        }
                        Command::TogglePointLabels => {
                            trigger_toggle_on!(
                                editor,
                                point_labels,
                                PointLabels,
                                command_info.command_mod.shift
                            );
                        }
                        Command::TogglePreviewMode => {
                            trigger_toggle_on!(
                                editor,
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
                        Command::DeleteSelection => {
                            editor.delete_selection();
                        }

                        _ => unreachable!(
                            "Command unimplemented!"
                        ),
                    }
                }

                Event::MouseMotion { x, y, .. } => {
                    let position = (x as f32, y as f32);
                    let meta = MouseInfo::new(&editor, None, position, None, keymod);
                    editor.dispatch_editor_event(EditorEvent::MouseEvent{
                        event_type: MouseEventType::Moved,
                        meta: meta

                    });

                    editor.mouse_info = meta;
                }

                Event::MouseButtonDown { mouse_btn, x, y, clicks: 2, .. } => {
                    
                    let position = (x as f32, y as f32);
                    let meta = MouseInfo::new(&editor, Some(mouse_btn), position, Some(true), keymod);              
                    editor.dispatch_editor_event(EditorEvent::MouseEvent{
                        event_type: MouseEventType::DoubleClick,
                        meta: meta
                    });

                    editor.mouse_info = meta;
                }

                Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                    
                    let position = (x as f32, y as f32);
                    let meta = MouseInfo::new(&editor, Some(mouse_btn), position, Some(true), keymod);              
                    editor.dispatch_editor_event(EditorEvent::MouseEvent{
                        event_type: MouseEventType::Pressed,
                        meta: meta
                    });

                    editor.mouse_info = meta;
                }

                Event::MouseButtonUp { mouse_btn, x, y, .. } => {
                    let position = (x as f32, y as f32);
                    let meta = MouseInfo::new(&editor, Some(mouse_btn), position, Some(false), keymod);
                    editor.dispatch_editor_event(EditorEvent::MouseEvent{
                        event_type: MouseEventType::Released,
                        meta: meta
                    });

                    editor.mouse_info = meta;
                }

                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::SizeChanged(x, y) => {
                        editor.viewport.winsize = (x as u32, y as u32);
                    }
                    WindowEvent::Resized(x, y) => {
                        editor.viewport.winsize = (x as u32, y as u32);
                    }

                    _ => {}
                },
                _ => {}
            }
        }

        editor.rebuild();

        // build and render imgui
        imgui_sdl2.prepare_frame(imgui.io_mut(), &editor.sdl_window.as_ref().unwrap(), &event_pump.mouse_state());
        let mut ui = imgui.frame();
        user_interface::build_imgui_ui(&mut editor, &mut ui);

        imgui_sdl2.prepare_render(&ui, &editor.sdl_window.as_ref().unwrap());
        let dd = ui.render();

        // draw glyph preview and imgui with skia
        let (window_width, window_height) = editor.sdl_window.as_ref().unwrap().vulkan_drawable_size();
        let extents = RafxExtents2D {
            width: window_width,
            height: window_height,
        };

        let drew = renderer.draw(extents, 1.0, |canvas, _coordinate_system_helper| {
            renderer::render_frame(&mut editor, canvas);
            imgui_renderer.render_imgui(canvas, dd);
        });

        if drew.is_err() {
            warn!("Failed to draw frame. This can happen when resizing due to VkError(ERROR_DEVICE_LOST); if happens otherwise, file an issue.");
        }
    }
}

fn initialize_sdl(v: &mut Editor, ws: &WindowSettings) -> (Sdl, Window) {
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

    let window = video_subsystem
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
        .expect("Failed to create editor.sdl_window.as_ref().unwrap()");

    /* TODO: Fix icon. 
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
    editor.sdl_window.as_ref().unwrap().set_icon(surface);
    */

    v.viewport.winsize = (WIDTH as u32, HEIGHT as u32);

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
