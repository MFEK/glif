for event in event_pump.poll_iter() {
    match event {
        Event::Quit { .. } => break 'main_loop,
        Event::KeyDown { keycode: Some(Keycode::Q), .. } => break 'main_loop,

        Event::KeyDown { keycode, keymod, ..} => {
            if let Some(vk) = keycode {
                events::console::set_state(vk, keymod);
            }
        }
        _ => {},
    }

    /*
    match event {
    Event::WindowEvent { event, .. } => match event {
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
                    Some(VirtualKeyCode::S) => {
                        if modifiers.ctrl() {
                            io::save_glif(v);
                        } else {
                            newmode = state::Mode::VWS;
                        }
                    }
                    Some(VirtualKeyCode::E) => {
                        if modifiers.ctrl() {
                            io::export_glif(v);
                        }
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
                    state::Mode::Select => {    events::select::mouse_moved(position, &v);
                                                events::vws::update_previews(position, &v)},
                    state::Mode::VWS => events::vws::mouse_moved(position, &v),
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
                    state::Mode::VWS => {
                        events::vws::mouse_button(position, &v, meta)
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
                            state::Mode::VWS => {
                                events::vws::mouse_pressed(position, &v, meta)
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
                            state::Mode::VWS => {
                                events::vws::mouse_released(position, &v, meta)
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
*/