use glifparser::glif::{HistoryType, HistoryEntry, MFEKPointData};

use super::Editor;

pub struct History {
    pub undo_stack: Vec<HistoryEntry<MFEKPointData>>,
    pub redo_stack: Vec<HistoryEntry<MFEKPointData>>,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: vec!(),
            redo_stack: vec!()
        }
    }
}

impl Editor {
    /// Pops a HistoryEntry off the layer stack and restores it.
    pub fn undo(&mut self) {
        let entry = self.history.undo_stack.pop();
        
        if let Some(undo_entry) = entry {
            self.history.redo_stack.push(HistoryEntry {
                description: "Undo".to_owned(),
                layer_idx: self.layer_idx,
                contour_idx: self.contour_idx,
                point_idx: self.point_idx,
                selected: Some(self.selected.clone()),
                layer: self.glyph.as_ref().unwrap().glif.layers[self.layer_idx.unwrap()].clone(),
                kind: HistoryType::LayerModified
            });
    
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

            self.rebuild_previews();
        }
    }

    pub fn redo(&mut self) {
        let entry = self.history.redo_stack.pop();
        
        if let Some(redo_entry) = entry {
            self.history.undo_stack.push(HistoryEntry {
                description: "Redo".to_owned(),
                layer_idx: self.layer_idx,
                contour_idx: self.contour_idx,
                point_idx: self.point_idx,
                selected: Some(self.selected.clone()),
                layer: self.glyph.as_ref().unwrap().glif.layers[self.layer_idx.unwrap()].clone(),
                kind: HistoryType::LayerModified
            });
    

            match redo_entry.kind {
                HistoryType::LayerModified => {
                    self.glyph.as_mut().unwrap().glif.layers[redo_entry.layer_idx.unwrap()] = redo_entry.layer;
                }
                HistoryType::LayerAdded => {
                    self.glyph.as_mut().unwrap().glif.layers.pop();

                }
                HistoryType::LayerDeleted => {
                    self.glyph.as_mut().unwrap().glif.layers.insert(redo_entry.layer_idx.unwrap(), redo_entry.layer);
                }
            }

            self.layer_idx = redo_entry.layer_idx;
            self.contour_idx = redo_entry.contour_idx;
            self.point_idx = redo_entry.point_idx;
            if let Some(selected) = redo_entry.selected {
                self.selected = selected
            }

            self.rebuild_previews();
        }    
    }
}