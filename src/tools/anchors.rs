use super::prelude::*;
use crate::tool_behaviors::pan::PanBehavior;
use crate::user_interface::{InputPrompt, Interface};
use crate::command::Command;

#[derive(Clone, Debug)]
pub struct Anchors {
    /// Selected anchor
    anchor_idx: Option<usize>,
}

impl Anchors {
    pub fn new() -> Self {
        Anchors {
            anchor_idx: None,
        }
    }
}

impl Tool for Anchors {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent { event_type, mouse_info } => {
                match event_type {
                    MouseEventType::Moved => { self.mouse_moved(v, mouse_info) }
                    MouseEventType::Pressed => { self.mouse_pressed(v, i, mouse_info) }
                    //MouseEventType::Released => { self.mouse_released(v, mouse_info) }
                    _ => {}
                }
            },
            EditorEvent::ToolCommand { command: Command::DeleteSelection, .. } => {
                self.delete_selection(v);
            },
            _ => {}
        }
    }

    fn draw(&self, v: &Editor, i: &Interface, canvas: &mut Canvas) {
        self.draw_selected(v, i, canvas);
    }

    fn ui(&mut self, v: &mut Editor, i: &mut Interface, ui: &mut Ui) {
        self.anchor_settings(v, i, ui);
    }
}

impl Anchors {
    fn draw_selected(&self, v: &Editor, i: &Interface, canvas: &mut Canvas) {
        if let Some(idx) = self.anchor_idx {
            let _scale = i.viewport.factor;
            v.with_glyph(|glif| {
                let anchor = &glif.anchors[idx];
                let mut paint = Paint::default();
                paint.set_color(SELECTED_ANCHOR_COLOR);
                canvas.draw_circle((calc_x(anchor.x), calc_y(anchor.y)), 2.5, &paint);
            });
        }
    }
}

// Make dialog box at right
impl Anchors {
    fn anchor_settings(&mut self, v: &mut Editor, i: &Interface, ui: &imgui::Ui) {
        let (tx, ty, tw, th) = i.get_tools_dialog_rect();
        imgui::Window::new(imgui::im_str!("Anchor Settings"))
            .bg_alpha(1.) // See comment on fn redraw_skia
            .flags(
                  imgui::WindowFlags::NO_RESIZE
                    | imgui::WindowFlags::NO_MOVE
                    | imgui::WindowFlags::NO_COLLAPSE,
            )
            .position(
                [tx, ty],
                imgui::Condition::Always,
            )
            .size(
                [tw, th],
                imgui::Condition::Always,
            )
            .build(ui, || {
                if let Some(idx) = self.anchor_idx {
                    v.with_glyph_mut(|glif| {
                        // X
                        let mut x = imgui::im_str!("{}", glif.anchors[idx].x);
                        let entered;
                        {
                        let it = ui.input_text(imgui::im_str!("X"), &mut x);
                        entered = it.enter_returns_true(true)
                            .chars_decimal(true)
                            .chars_noblank(true)
                            .auto_select_all(true)
                            .build();
                        }
                        if entered {
                            if x.to_str().len() > 0 {
                                let new_x: f32 = x.to_str().parse().unwrap();
                                glif.anchors[idx].x = new_x;
                            }
                        }
                        // Y
                        let mut y = imgui::im_str!("{}", glif.anchors[idx].y);
                        let entered;
                        {
                        let it = ui.input_text(imgui::im_str!("Y"), &mut y);
                        entered = it.enter_returns_true(true)
                            .chars_decimal(true)
                            .chars_noblank(true)
                            .auto_select_all(true)
                            .build();
                        }
                        if entered {
                            if y.to_str().len() > 0 {
                                let new_y: f32 = y.to_str().parse().unwrap();
                                glif.anchors[idx].y = new_y;
                            }
                        }
                        // Class
                        let mut class = imgui::im_str!("{}", &glif.anchors[idx].class);
                        let entered;
                        {
                        let it = ui.input_text(imgui::im_str!("Class"), &mut class);
                        entered = it.enter_returns_true(true)
                            .chars_noblank(true)
                            .auto_select_all(true)
                            .build();
                        }
                        if entered {
                            if class.to_str().len() > 0 {
                                glif.anchors[idx].class = class.to_str().to_string();
                            }
                        }
                    });
                }
            });
    }
}

// Mouse
use std::rc::Rc;
use crate::editor::{Editor};
use glifparser::Anchor as GlifAnchor;
use skulpin::skia_safe::{Paint};
impl Anchors {
    fn mouse_pressed(&mut self, v: &mut Editor, i: &mut Interface, mouse_info: MouseInfo) {
        // if the user clicked middle mouse we initiate a pan behavior
        if mouse_info.button == MouseButton::Middle {
            v.set_behavior(Box::new(PanBehavior::new(i.viewport.clone(), mouse_info)));
            return
        }

        // Reset selected anchor
        self.anchor_idx = None;

        // Find if we've clicked on an anchor
        v.with_glyph(|glif| {
            for (idx, anchor) in glif.anchors.iter().enumerate() {
                let size = ((ANCHOR_RADIUS * 2.) + (ANCHOR_STROKE_THICKNESS * 2.)) * (1. / i.viewport.factor);
                // Topleft corner of point
                let anchor_tl = SkPoint::new(
                    calc_x(anchor.x as f32) - (size / 2.),
                    calc_y(anchor.y as f32) - (size / 2.),
                );
                let anchor_rect = SkRect::from_point_and_size(anchor_tl, (size, size));
                let sk_mpos = SkPoint::new(mouse_info.position.0 as f32, mouse_info.position.1 as f32);
                if anchor_rect.contains(sk_mpos) {
                    self.anchor_idx = Some(idx);
                    break;
                }
            }
        });

        // If we have, return, and wait for motion
        if let Some(_idx) = self.anchor_idx { return }

        let position = i.mouse_info.position;
        i.push_prompt(InputPrompt::Text {
            label: "Anchor name:".to_string(),
            default: "".to_string(),
            func: Rc::new(move |v, string| {
                v.with_glyph_mut(|glif| {
                    let mut anchor = GlifAnchor::new();
                    anchor.x = f32::floor(calc_x(position.0));
                    anchor.y = f32::floor(calc_y(position.1));
                    anchor.class = string.clone();
                    glif.anchors.push(anchor);
                });
            }),
        });
    }

    fn mouse_moved(&mut self, v: &mut Editor, mouse_info: MouseInfo) {
        if !mouse_info.is_down { return }

        v.with_glyph_mut(|glif| {
            if let Some(idx) = self.anchor_idx {
                // Anchors can't be non-integers in OT spec
                glif.anchors[idx].x = f32::floor(calc_x(mouse_info.position.0));
                glif.anchors[idx].y = f32::floor(calc_y(mouse_info.position.1));
            }
        });
    }
}

// Keyed
impl Anchors {
    fn delete_selection(&mut self, v: &mut Editor) {
        if let Some(idx) = self.anchor_idx {
            v.with_glyph_mut(|glif| {
                glif.anchors.remove(idx);
            });
        }
        self.anchor_idx = None;
    }
}
