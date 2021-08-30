//! MFEKglif - A cross-platform .glif renderer and editor.
//! (c) 2020–2021 Fredrick R. Brennan, Matthew Blanchard & MFEK Authors
//! Apache 2.0 licensed. See AUTHORS.
#![allow(non_snake_case)] // for our name MFEKglif

use crate::{constants::OFFSET_FACTOR, tools::zoom::{zoom_in_factor, zoom_out_factor}};
use command::{Command, CommandInfo, CommandMod};
use editor::{Editor};
use glifrenderer::toggles::{PointLabels, PreviewMode};
use tools::{EditorEvent, MouseEventType, ToolEnum};
use user_interface::{ImguiManager, Interface};
use util::argparser::HeadlessMode;

use sdl2::event::{Event, WindowEvent};
pub use skulpin::{rafx::api as RafxApi, skia_safe};

use crate::user_interface::mouse_input::MouseInfo;

use enum_unitary::IntoEnumIterator;

mod command;
pub mod constants;
mod contour_operations;
pub mod editor;
mod filedialog;
mod io;
mod ipc;
pub mod settings;
mod system_fonts;
mod tool_behaviors;
mod tools;
mod user_interface;
pub mod util;
mod render;

fn main() {
    util::init_env_logger();
    util::set_panic_hook();

    let args = util::argparser::parse_args();

    let mut editor = Editor::new();

    if args.headless_mode != HeadlessMode::None {
        editor.headless(&args); // this function always calls exit()
    }

    let filename = filedialog::filename_or_panic(&args.filename, Some("glif"), None);
    let mut interface = Interface::new(filename.to_str().unwrap());
    let mut imgui_manager = ImguiManager::new(&interface.sdl_window);

    let mut skulpin_renderer = Interface::initialize_skulpin_renderer(&interface.sdl_window);

    // Makes glyph available to on_load_glif events
    io::load_glif(&mut editor, &mut interface, &filename);

    command::initialize_keybinds();
    // TODO: Replace console! tools::console::initialize_console_commands();

    let mut event_pump = interface.get_event_pump();
    'main_loop: loop {
        // Quit from console
        if editor.quit_requested {
            break 'main_loop;
        }

        let keys_down = interface.get_pressed_keys(&event_pump);
        let keymod = command::keys_down_to_mod(&keys_down).unwrap_or(CommandMod::none());

        // sdl event handling
        for event in event_pump.poll_iter() {
            util::debug_event!("Got event: {:?}", &event);

            match &event {
                Event::Quit { .. } => break 'main_loop,
                _ => {}
            }

            if imgui_manager.handle_imgui_event(&event) {
                continue;
            }
            if interface.active_prompts() {
                continue;
            }

            // we're gonna handle console text input here as this should steal input from the command system
            /* TODO: Replace console!
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
            */

            match event {
                Event::KeyDown {
                    keycode, keymod, ..
                } => {
                    // we don't care about keydown events that have no keycode
                    if !keycode.is_some() {
                        continue;
                    }
                    let keycode = keycode.unwrap();

                    /* TODO: Replace console!
                    tools::console::set_state(&mut editor, &mut interface, keycode, keymod);
                    if CONSOLE.with(|c| c.borrow_mut().active) {
                        continue;
                    } */

                    // check if we've got a command
                    let command_info: CommandInfo =
                        match command::keycode_to_command(&keycode, &keys_down) {
                            Some(command) => command,
                            None => continue,
                        };

                    let mut delete_after = false;
                    editor.dispatch_editor_event(
                        &mut interface,
                        EditorEvent::ToolCommand {
                            command: command_info.command,
                            command_mod: command_info.command_mod,
                            stop_after: &mut delete_after,
                        },
                    );
                    if delete_after {
                        continue;
                    }

                    match command_info.command {
                        Command::ResetScale => {
                            interface.update_viewport(None, Some(1.));
                        }
                        Command::ZoomIn => {
                            let scale = zoom_in_factor(interface.viewport.factor, &mut interface);
                            interface.update_viewport(None, Some(scale));
                        }
                        Command::ZoomOut => {
                            let scale = zoom_out_factor(interface.viewport.factor, &mut interface);
                            interface.update_viewport(None, Some(scale));
                        }
                        Command::NudgeUp => {
                            interface.update_viewport(Some((0., OFFSET_FACTOR)), None);
                        }
                        Command::NudgeDown => {
                            interface.update_viewport(Some((0., -OFFSET_FACTOR)), None);
                        }
                        Command::NudgeLeft => {
                            interface.update_viewport(Some((OFFSET_FACTOR, 0.)), None);
                        }
                        Command::NudgeRight => {
                            interface.update_viewport(Some((-OFFSET_FACTOR, 0.)), None);
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
                        /*
                        Command::ToolVWS => {
                            editor.set_tool(ToolEnum::VWS);
                        }
                        Command::ToolMeasure => {
                            editor.set_tool(ToolEnum::Measure);
                        }
                        */
                        Command::ToolAnchors => {
                            editor.set_tool(ToolEnum::Anchors);
                        }
                        /*
                        Command::ToolShapes => {
                            editor.set_tool(ToolEnum::Shapes);
                        }
                        */
                        Command::TogglePointLabels => {
                            trigger_toggle_on!(
                                interface,
                                point_labels,
                                PointLabels,
                                !command_info.command_mod.shift
                            );
                        }
                        Command::TogglePreviewMode => {
                            trigger_toggle_on!(
                                interface,
                                preview_mode,
                                PreviewMode,
                                !command_info.command_mod.shift
                            );
                        }
                        /* TODO: Replace console!
                        Command::ToggleConsole => {
                            CONSOLE.with(|c| {
                                c.borrow_mut().active = true;
                            });
                        }*/
                        Command::DeleteSelection => {
                            editor.delete_selection();
                        }
                        Command::SelectAll => {} // handled by select tool, only when select active
                        Command::CopySelection => {
                            editor.copy_selection();
                        }
                        Command::PasteSelection => {
                            editor.paste_selection(interface.mouse_info.position);
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
                            io::load_glif(&mut editor, &mut interface, &filename);
                        }
                        Command::IOSave => {
                            drop(editor.save_glif(false));
                        }
                        Command::IOSaveAs => match editor.save_glif(true) {
                            Ok(pb) => io::load_glif(&mut editor, &mut interface, &pb),
                            Err(()) => {}
                        },
                        Command::IOFlatten => {
                            editor.flatten_glif(true);
                        }
                        Command::IOExport => {
                            editor.export_glif();
                        }
                        Command::Quit => {
                            break 'main_loop;
                        }
                        // TODO: More elegantly deal with Command's meant for consumption by a
                        // single tool?
                        Command::ReverseContour => {
                            log::warn!("Tried to reverse contour outside Select tool");
                        }
                        Command::SkiaDump => {
                            editor.skia_dump();
                        }
                        #[allow(unreachable_patterns)]
                        // This failsafe is here if you add a Command.
                        cmd => unreachable!("Command unimplemented: {:?}", cmd),
                    }
                }

                Event::MouseMotion { x, y, .. } => {
                    let position = (x as f32, y as f32);
                    let mouse_info = MouseInfo::new(&interface, None, position, None, keymod);
                    editor.dispatch_editor_event(
                        &mut interface,
                        EditorEvent::MouseEvent {
                            event_type: MouseEventType::Moved,
                            mouse_info,
                        },
                    );

                    interface.mouse_info = mouse_info;
                }

                Event::MouseButtonDown {
                    mouse_btn,
                    x,
                    y,
                    clicks: 2,
                    ..
                } => {
                    let position = (x as f32, y as f32);
                    let mouse_info =
                        MouseInfo::new(&interface, Some(mouse_btn), position, Some(true), keymod);
                    editor.dispatch_editor_event(
                        &mut interface,
                        EditorEvent::MouseEvent {
                            event_type: MouseEventType::DoubleClick,
                            mouse_info,
                        },
                    );

                    interface.mouse_info = mouse_info;
                }

                Event::MouseButtonDown {
                    mouse_btn, x, y, ..
                } => {
                    let position = (x as f32, y as f32);
                    let mouse_info =
                        MouseInfo::new(&interface, Some(mouse_btn), position, Some(true), keymod);
                    editor.dispatch_editor_event(
                        &mut interface,
                        EditorEvent::MouseEvent {
                            event_type: MouseEventType::Pressed,
                            mouse_info,
                        },
                    );

                    interface.mouse_info = mouse_info;
                }

                Event::MouseButtonUp {
                    mouse_btn, x, y, ..
                } => {
                    let position = (x as f32, y as f32);
                    let mouse_info =
                        MouseInfo::new(&interface, Some(mouse_btn), position, Some(false), keymod);
                    editor.dispatch_editor_event(
                        &mut interface,
                        EditorEvent::MouseEvent {
                            event_type: MouseEventType::Released,
                            mouse_info,
                        },
                    );

                    interface.mouse_info = mouse_info;
                }

                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::SizeChanged(x, y) => {
                        interface.viewport.winsize = (x as f32, y as f32);
                    }
                    WindowEvent::Resized(x, y) => {
                        interface.viewport.winsize = (x as f32, y as f32);
                    }

                    _ => {}
                },
                _ => {}
            }
        }

        editor.rebuild();
        interface.render(
            &mut editor,
            &mut imgui_manager.imgui_context,
            &mut imgui_manager.imgui_sdl2,
            &mut imgui_manager.imgui_renderer,
            &mut skulpin_renderer,
            &event_pump.mouse_state(),
        );
    }
}
