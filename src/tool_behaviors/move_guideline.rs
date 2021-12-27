use super::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct MoveGuideline {
    pub selected_idx: usize,
    pub global: bool,
    pub mouse_info: MouseInfo,
}

impl MoveGuideline {
    pub fn mouse_moved(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        if !v.is_modifying() {
            v.begin_modification("Move guideline.");
        }

        let mp = mouse_info.position;
        let selected = self.selected_idx;

        if self.global {
            let gl_len = v.with_glyph(|g| g.guidelines.len());
            let guideline = &mut v.guidelines[selected - gl_len];
            if guideline.data.as_guideline().fixed {
                self.mouse_info = mouse_info;
                return;
            }
            guideline.at.x = mp.0;
            guideline.at.y = mp.1;

            if guideline.data.as_guideline().right {
                v.with_glyph_mut(|glyph| {
                    if let Some(w) = glyph.width.as_mut() {
                        *w = mp.0 as u64;
                    }
                });
                v.add_width_guidelines();
            }
        } else {
            v.with_glyph_mut(|glyph| {
                glyph.guidelines[selected].at.x = mp.0;
                glyph.guidelines[selected].at.y = mp.1;
            });
        }

        self.mouse_info = mouse_info;
    }

    pub fn mouse_released(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        if mouse_info.button == self.mouse_info.button {
            v.mark_dirty(); // Moving a guideline technically is an "empty" mod otherwise.
            if v.is_modifying() {
                v.end_modification();
            } else {
                log::debug!("Didn't move the guideline.");
            }
            v.pop_behavior();
        }
    }
}

impl ToolBehavior for MoveGuideline {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent {
                event_type,
                mouse_info,
            } => match event_type {
                MouseEventType::Released => self.mouse_released(v, i, mouse_info),
                MouseEventType::Moved => self.mouse_moved(v, i, mouse_info),
                _ => {}
            },
            _ => {}
        }
    }
}
