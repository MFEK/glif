use glifparser::glif::HistoryType;

use super::Editor;

impl Editor {
    /// Pops a HistoryEntry off the layer stack and restores it.
    pub fn undo(&mut self) {
        let entry = self.history.pop();
        
        if let Some(undo_entry) = entry {
            match undo_entry.kind {
                HistoryType::LayerModified => {
                    self.glyph.as_mut().unwrap().glif.layers[undo_entry.layer_idx.unwrap()] = undo_entry.layer;
                }
                HistoryType::LayerAdded => {
                    self.glyph.as_mut().unwrap().glif.layers.pop();

                }
                HistoryType::LayerDeleted => {
                    self.glyph.as_mut().unwrap().glif.layers.insert(undo_entry.layer_idx.unwrap(), undo_entry.layer);
                }
            }

            self.layer_idx = undo_entry.layer_idx;
            self.contour_idx = undo_entry.contour_idx;
            self.point_idx = undo_entry.point_idx;
            if let Some(selected) = undo_entry.selected {
                self.selected = selected
            }
        }
    }

    pub fn redo() {
        // TODO:
    }
}