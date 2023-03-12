use super::prelude::*;
use crate::command::Command;
use crate::tool_behaviors::{zoom_scroll::ZoomScroll};
use crate::user_interface::{InputPrompt, Interface};

mod dialog;

#[derive(Clone, Debug)]
pub struct Anchors {
    /// Selected anchor
    anchor_idx: Option<usize>,

    // for it's text dialog
    edit_buf: HashMap<String, String>
}

impl Anchors {
    pub fn new() -> Self {
        Anchors { 
            anchor_idx: None,
            edit_buf: HashMap::new()
        }
    }
}

impl Tool for Anchors {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent {
                event_type,
                mouse_info,
            } => match event_type {
                MouseEventType::Moved => self.mouse_moved(v, mouse_info),
                MouseEventType::Pressed => self.mouse_pressed(v, i, mouse_info),
                MouseEventType::Released => self.mouse_released(v),
                _ => {}
            },
            EditorEvent::ToolCommand {
                command: Command::DeleteSelection,
                ..
            } => {
                self.delete_selection(v);
            }
            EditorEvent::ScrollEvent { .. } => ZoomScroll::default().event(v, i, event),
            #[allow(unreachable_patterns)] // more events likely to be added.
            _ => {}
        }
    }

    fn draw(&mut self, v: &Editor, i: &Interface, canvas: &mut Canvas) {
        self.draw_selected(v, i, canvas);
    }

    fn dialog(&mut self, v: &mut Editor, i: &mut Interface, ui: &mut Ui) -> bool {
        if let Some(_) = self.anchor_idx {
            self.anchor_settings(v, i, ui);
            return true;
        }

        false
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
                canvas.draw_circle((anchor.x, anchor.y), 2.5, &paint);
            });
        }
    }
}


// Mouse
use crate::editor::Editor;
use glifparser::Anchor as GlifAnchor;
use skia_safe::Paint;
use std::collections::HashMap;
use std::rc::Rc;
impl Anchors {
    fn mouse_pressed(&mut self, v: &mut Editor, i: &mut Interface, mouse_info: MouseInfo) {
        // Reset selected anchor
        self.anchor_idx = None;

        // Find if we've clicked on an anchor
        v.with_glyph(|glif| {
            for (idx, anchor) in glif.anchors.iter().enumerate() {
                let size = ((ANCHOR_RADIUS * 2.) + (ANCHOR_STROKE_THICKNESS * 2.))
                    * (1. / i.viewport.factor);
                // Topleft corner of point
                let anchor_tl =
                    SkPoint::new(anchor.x as f32 - (size / 2.), anchor.y as f32 - (size / 2.));
                let anchor_rect = SkRect::from_point_and_size(anchor_tl, (size, size));
                let sk_mpos =
                    SkPoint::new(mouse_info.position.0 as f32, mouse_info.position.1 as f32);
                if anchor_rect.contains(sk_mpos) {
                    self.anchor_idx = Some(idx);
                    break;
                }
            }
        });

        // If we have, return, and wait for motion
        if let Some(_idx) = self.anchor_idx {
            return;
        }

        let position = i.mouse_info.position;
        i.push_prompt(InputPrompt::Text {
            label: "Anchor name:".to_string(),
            default: "".to_string(),
            func: Rc::new(move |v, string| {
                if string.is_empty() {
                    return;
                }
                v.begin_modification("Add anchor.", false);
                v.with_glyph_mut(|glif| {
                    let mut anchor = GlifAnchor::default();
                    anchor.x = f32::floor(position.0);
                    anchor.y = f32::floor(position.1);
                    anchor.class = Some(string.clone());
                    glif.anchors.push(anchor);
                });
                v.end_modification();
            }),
        });
    }

    fn mouse_moved(&mut self, v: &mut Editor, mouse_info: MouseInfo) {
        if let Some(idx) = self.anchor_idx {
            if !mouse_info.is_down {
                return;
            }

            if !v.is_modifying() {
                v.begin_modification("Move anchor.", false);
            }

            v.with_glyph_mut(|glif| {
                // Anchors can't be non-integers in OT spec
                glif.anchors[idx].x = f32::floor(mouse_info.position.0);
                glif.anchors[idx].y = f32::floor(mouse_info.position.1);
            });
        }
    }

    fn mouse_released(&mut self, v: &mut Editor) {
        v.end_modification();
    }
}

// Keyed
impl Anchors {
    fn delete_selection(&mut self, v: &mut Editor) {
        if let Some(idx) = self.anchor_idx {
            v.begin_modification("Delete anchor.", false);
            v.with_glyph_mut(|glif| {
                glif.anchors.remove(idx);
            });
            v.end_modification();
        }
        self.anchor_idx = None;
    }
}
