//! MFEKglif - A cross-platform .glif renderer and editor.
//! (c) 2020–2021 Fredrick R. Brennan, Matthew Blanchard & MFEK Authors
//! Apache 2.0 licensed. See AUTHORS.
#![allow(non_snake_case)] // for our name MFEKglif

use command::{Command, CommandInfo, CommandMod};
use tools::{EditorEvent, MouseEventType, ToolEnum};
use editor::{Editor, MouseInfo, HandleStyle, PointLabels, PreviewMode, CONSOLE};
use util::argparser::HeadlessMode;

use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    video::Window,
    Sdl,
};
pub use skulpin::{skia_safe, rafx::api as RafxApi};
use imgui_skia_renderer::Renderer;

use enum_iterator::IntoEnumIterator as _;

use std::collections::HashSet;

pub mod editor;
mod filedialog;
pub mod util;
mod tools;
mod command;
mod io;
mod ipc;
mod renderer;
pub mod settings;
mod system_fonts;
mod user_interface;
mod contour_operations;
mod window;

fn main() {
    util::init_env_logger();
    util::set_panic_hook();

    let args = util::argparser::parse_args();

    let mut editor = Editor::new();

    if args.headless_mode != HeadlessMode::None {
        editor.headless(&args); // this function always calls exit()
    }

    let filename = filedialog::filename_or_panic(&args.filename, Some("glif"), None);

    let (sdl_context, sdl_window): (Sdl, Window) = window::initialize_sdl(&mut editor, filename.to_str().unwrap());

    editor.sdl_context = Some(sdl_context);
    editor.sdl_window = Some(sdl_window);

    // Skulpin initialization TODO: proper error handling
    let mut renderer = window::initialize_skulpin_renderer(&editor.sdl_window.as_ref().unwrap()).unwrap();

    // set up imgui
    let mut imgui = user_interface::setup_imgui();
    let mut imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &editor.sdl_window.as_ref().unwrap());
    let imgui_renderer = Renderer::new(&mut imgui);

    let mut event_pump = editor.sdl_context
        .as_ref()
        .unwrap()
        .event_pump()
        .expect("Could not create sdl event pump");
 
    // Makes glyph available to on_load_glif events
    io::load_glif(&mut editor, &filename);

    command::initialize_keybinds();
    tools::console::initialize_console_commands();

    'main_loop: loop {
        // Quit from console
        if editor.quit_requested { break 'main_loop }

        // Create a set of pressed Keys.
        let keys_down: HashSet<Keycode> = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        let keymod = command::keys_down_to_mod(&keys_down).unwrap_or(CommandMod::none());

        // sdl event handling
        for event in event_pump.poll_iter() {
            util::debug_event!("Got event: {:?}", &event);
            imgui_sdl2.handle_event(&mut imgui, &event);
            if imgui_sdl2.ignore_event(&event) {
                continue;
            };

            match &event {
                Event::Quit { .. } => break 'main_loop,
                _ => {}
            }

            if !editor.prompts.is_empty() { continue; }
            // we're gonna handle console text input here as this should steal input from the command system
            match &event {
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
                    if delete_after { continue; }

                    use crate::renderer::constants::OFFSET_FACTOR;
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
                        Command::SelectAll => {} // handled by select tool, only when select active
                        Command::CopySelection => {
                            editor.copy_selection();
                        }
                        Command::PasteSelection => {
                            editor.paste_selection(editor.mouse_info.position);
                        }
                        Command::CutSelection => {
                            editor.copy_selection();
                            editor.delete_selection();
                        }
                        Command::HistoryUndo => {
                            editor.undo();
                        }
                        Command::HistoryRedo => {
                            editor.redo();
                        }
                        Command::IOOpen => {
                            let filename = match filedialog::open_filename(Some("glif"), None) {
                                Some(f) => f,
                                None => continue,
                            };
                            io::load_glif(&mut editor, &filename);
                        }
                        Command::IOSave => {
                            drop(editor.save_glif(false));
                        }
                        Command::IOSaveAs => {
                            match editor.save_glif(true) {
                                Ok(pb) => io::load_glif(&mut editor, &pb),
                                Err(()) => {},
                            }
                        }
                        Command::IOFlatten => {
                            editor.flatten_glif(true);
                        }
                        Command::IOExport => {
                            editor.export_glif();
                        }
                        Command::Quit => {
                            break 'main_loop;
                        }
                        Command::SkiaDump => {
                            editor.skia_dump();
                        }
                        #[allow(unreachable_patterns)] // This failsafe is here if you add a Command.
                        cmd => unreachable!(
                            "Command unimplemented: {:?}", cmd
                        ),
                    }
                }

                Event::MouseMotion { x, y, .. } => {
                    let position = (x as f32, y as f32);
                    let meta = MouseInfo::new(&editor, None, position, None, keymod);
                    editor.dispatch_editor_event(EditorEvent::MouseEvent{
                        event_type: MouseEventType::Moved,
                        meta,

                    });

                    editor.mouse_info = meta;
                }

                Event::MouseButtonDown { mouse_btn, x, y, clicks: 2, .. } => {
                    
                    let position = (x as f32, y as f32);
                    let meta = MouseInfo::new(&editor, Some(mouse_btn), position, Some(true), keymod);              
                    editor.dispatch_editor_event(EditorEvent::MouseEvent{
                        event_type: MouseEventType::DoubleClick,
                        meta,
                    });

                    editor.mouse_info = meta;
                }

                Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                    
                    let position = (x as f32, y as f32);
                    let meta = MouseInfo::new(&editor, Some(mouse_btn), position, Some(true), keymod);              
                    editor.dispatch_editor_event(EditorEvent::MouseEvent{
                        event_type: MouseEventType::Pressed,
                        meta,
                    });

                    editor.mouse_info = meta;
                }

                Event::MouseButtonUp { mouse_btn, x, y, .. } => {
                    let position = (x as f32, y as f32);
                    let meta = MouseInfo::new(&editor, Some(mouse_btn), position, Some(false), keymod);
                    editor.dispatch_editor_event(EditorEvent::MouseEvent{
                        event_type: MouseEventType::Released,
                        meta,
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
        let extents = RafxApi::RafxExtents2D {
            width: window_width,
            height: window_height,
        };

        let drew = renderer.draw(extents, 1.0, |canvas, _coordinate_system_helper| {
            renderer::render_frame(&mut editor, canvas);
            imgui_renderer.render_imgui(canvas, dd);
        });

        if drew.is_err() {
            log::warn!("Failed to draw frame. This can happen when resizing due to VkError(ERROR_DEVICE_LOST); if happens otherwise, file an issue.");
        }
    }
}
