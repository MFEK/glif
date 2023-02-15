mod dialog;

use std::collections::HashMap;
use std::rc::Rc;

use glifparser::glif::contour_operations::ContourOperations;
use glifparser::glif::contour_operations::pap::{PAPContour, PatternCopies, PatternSubdivide, PatternStretch};

use super::prelude::*;
use crate::tool_behaviors::zoom_scroll::ZoomScroll;
use crate::{editor::Editor, user_interface::InputPrompt};

#[derive(Clone, Debug)]
pub struct PAP {
    pattern_layer: Option<usize>,
    edit_buf: HashMap<String, String>
}

impl Tool for PAP {
    #[rustfmt::skip]
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::ScrollEvent { .. } => ZoomScroll::default().event(v, i, event),
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
        }
    }
}
