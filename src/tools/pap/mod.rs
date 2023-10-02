mod dialog;

use std::collections::HashMap;

use super::prelude::*;
use crate::tool_behaviors::zoom_scroll::ZoomScroll;
use crate::tools::Select;
use crate::editor::Editor;

#[derive(Clone, Debug)]
pub struct PAP {
    pattern_layer: Option<usize>,
    edit_buf: HashMap<String, String>,
    select_tool: Select
}

impl Tool for PAP {
    #[rustfmt::skip]
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::ScrollEvent { .. } => ZoomScroll::default().event(v, i, event),
            EditorEvent::MouseEvent {
                event_type,
                mouse_info,
            } => match event_type {
                MouseEventType::Pressed => self.select_tool.mouse_pressed(v, i, mouse_info),
                MouseEventType::DoubleClick => self.select_tool.mouse_double_pressed(v, i, mouse_info),
                _ => {}
            },
            _ => {}
        }
    }

    fn dialog(&mut self, v: &mut Editor, i: &mut Interface, ui: &mut Ui) -> bool {
        let show_dialog = match v.contour_idx {
            Some(_) => true,
            _ => false,
        };

        if show_dialog {
            self.tool_dialog(v, i, ui);
        }

        return show_dialog;
    }
}

impl PAP {
    pub fn new() -> Self {
        Self {
            edit_buf: HashMap::new(),
            pattern_layer: None,
            select_tool: Select::new()
        }
    }
}
