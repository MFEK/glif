use super::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct MoveGuideline {
    pub selected_idx: usize,
    pub global: bool,
    pub mouse_info: MouseInfo,
}

impl MoveGuideline {
    pub fn mouse_moved(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        let mp = self.mouse_info.position;
        let delta = (mp.0 - mouse_info.position.0, mp.1 - mouse_info.position.1);
        let selected = self.selected_idx;

        if !v.is_modifying() {
            v.begin_modification("Move guideline.");
        }

        if self.global {
            let gl_len = v.with_glyph(|g| g.guidelines.len());
            let guideline = &mut v.guidelines[selected - gl_len];
            if guideline.name.as_ref().unwrap() != "lbearing" {
                guideline.at.x -= delta.0;
                guideline.at.y += delta.1;
            }
        } else {
            v.with_glyph_mut(|glyph| {
                glyph.guidelines[selected].at.x -= delta.0;
                glyph.guidelines[selected].at.y += delta.1;
            });
        }

        self.mouse_info = mouse_info;
    }

    pub fn mouse_released(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        if mouse_info.button == self.mouse_info.button {
            v.end_modification();
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
