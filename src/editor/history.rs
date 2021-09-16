use glifparser::glif::HistoryEntry;

use super::Editor;

pub struct History {
    pub undo_stack: Vec<HistoryEntry>,
    pub redo_stack: Vec<HistoryEntry>,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: vec!(),
            redo_stack: vec!()
        }
    }
}

impl History {
    pub fn add_undo_entry(&mut self, entry: HistoryEntry ) {
        log::debug!("Added undo entry: {0}", entry.description);
        self.undo_stack.push(entry);
        self.redo_stack.clear();
    }
}

impl Editor {
    /// Pops a HistoryEntry off the layer stack and restores it.
    pub fn undo(&mut self) {
        if self.modifying { return; }
        let entry = self.history.undo_stack.pop();
        
        if let Some(undo_entry) = entry {
            self.history.redo_stack.push( HistoryEntry {
                description: "Undo".to_owned(),
                layer_idx: self.layer_idx,
                contour_idx: self.contour_idx,
                point_idx: self.point_idx,
                selected: Some(self.selected.clone()),
                glyph: self.glyph.as_ref().unwrap().clone(),
            });
    
            self.glyph = Some(undo_entry.glyph.clone());
            self.layer_idx = undo_entry.layer_idx;
            self.contour_idx = undo_entry.contour_idx;
            self.point_idx = undo_entry.point_idx;
            if let Some(selected) = undo_entry.selected {
                self.selected = selected
            }

            self.mark_preview_dirty();
        }
    }

    pub fn redo(&mut self) {
        if self.modifying { return; }
        let entry = self.history.redo_stack.pop();
        
        if let Some(redo_entry) = entry {
            self.history.undo_stack.push(HistoryEntry {
                description: "Redo".to_owned(),
                layer_idx: self.layer_idx,
                contour_idx: self.contour_idx,
                point_idx: self.point_idx,
                selected: Some(self.selected.clone()),
                glyph: self.glyph.as_ref().unwrap().clone(),
            });
    

            self.glyph = Some(redo_entry.glyph.clone());
            self.layer_idx = redo_entry.layer_idx;
            self.contour_idx = redo_entry.contour_idx;
            self.point_idx = redo_entry.point_idx;
            if let Some(selected) = redo_entry.selected {
                self.selected = selected
            }

            self.mark_preview_dirty();
        }    
    }
}
