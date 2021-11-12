use super::prelude::*;

#[derive(Clone, Debug)]
pub struct PanBehavior {
    viewport: Viewport,
    mouse_info: MouseInfo,
}

impl PanBehavior {
    pub fn new(viewport: Viewport, mouse_info: MouseInfo) -> Self {
        PanBehavior {
            viewport,
            mouse_info,
        }
    }

    pub fn mouse_moved(&mut self, _v: &mut Editor, i: &mut Interface, mouse_info: MouseInfo) {
        let mut new_offset = self.viewport.offset;
        new_offset.0 +=
            (mouse_info.raw_absolute_position.0 - self.mouse_info.raw_absolute_position.0).floor();
        new_offset.1 +=
            (mouse_info.raw_absolute_position.1 - self.mouse_info.raw_absolute_position.1).floor();

        i.viewport.offset = new_offset;
    }

    pub fn mouse_released(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        if mouse_info.button == self.mouse_info.button {
            v.pop_behavior();
        }
    }
}

#[rustfmt::skip]
impl ToolBehavior for PanBehavior {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        if let EditorEvent::MouseEvent { mouse_info, event_type } = event {
            match event_type {
                MouseEventType::Released => self.mouse_released(v, i, mouse_info),
                MouseEventType::Moved => self.mouse_moved(v, i, mouse_info),
                _ => (),
            }
        }
    }
}
