use super::prelude::*;

use crate::tools::zoom::{zoom_in_factor, zoom_out_factor};

#[derive(Clone, Debug, Default)]
pub struct ZoomScroll;

impl ZoomScroll {
    fn scroll(&self, _v: &mut Editor, i: &mut Interface, vertical: i32) {
        let zoomout = vertical < 0;
        for _ in 0..vertical.abs() {
            let scale = if zoomout {
                zoom_out_factor(i)
            } else {
                zoom_in_factor(i)
            };
            i.update_viewport(None, Some(scale));
        }
    }
}

impl ToolBehavior for ZoomScroll {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        if let EditorEvent::ScrollEvent { vertical, .. } = event {
            self.scroll(v, i, vertical);
        }
    }
}
