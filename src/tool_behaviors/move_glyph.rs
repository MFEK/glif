use super::prelude::*;

#[derive(Clone, Default, derivative::Derivative)]
#[derivative(Debug)]
pub struct MoveGlyph {
    mouse_info: Option<MouseInfo>,
    #[derivative(Debug="ignore")]
    glyph: Option<MFEKGlif<MFEKGlifPointData>>,
}

impl MoveGlyph {
    pub fn mouse_pressed(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        v.begin_modification("Move glyph.");

        self.mouse_info = Some(mouse_info);
        self.glyph = Some(v.with_glyph(|glyph|glyph.clone()));
    }

    pub fn mouse_moved(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        let (mp, mg_glyph) = if let (Some(mi), Some(g)) = (self.mouse_info.as_mut(), self.glyph.as_mut()) {
            (mi.position, g)
        } else {
            return;
        };

        let delta = (mp.0 - mouse_info.position.0, mp.1 - mouse_info.position.1);

        v.with_glyph_mut(|glyph| {
            *glyph = mg_glyph.clone();
            if let Some(w) = glyph.width.as_mut() {
                *w = (*w as f32 - delta.0) as u64;
            }
        });
        v.add_width_guidelines();

        editor::util::move_all_layers(v, delta.0, 0.);
    }

    pub fn mouse_released(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        if let Some(mi) = self.mouse_info {
            if mouse_info.button == mi.button {
                v.end_modification();
                self.glyph = None;
                v.pop_behavior();
            }
        }
    }
}

impl ToolBehavior for MoveGlyph {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent {
                event_type,
                mouse_info,
            } => match event_type {
                MouseEventType::Pressed => self.mouse_pressed(v, i, mouse_info),
                MouseEventType::Released => self.mouse_released(v, i, mouse_info),
                MouseEventType::Moved => self.mouse_moved(v, i, mouse_info),
                _ => {}
            },
            _ => {}
        }
    }
}
