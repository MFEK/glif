use glifparser::glif::{MFEKOutline, HistoryEntry, HistoryType, Layer};
use super::Editor;

impl Editor {
    /// Adds a new layer. This generates a HistoryEntry and sets the selection to point to the newly created
    /// layer.
    pub fn new_layer(&mut self) {
        let new_layer = Layer {
            name: format!("{}", self.glyph.as_ref().unwrap().layers.len()),
            visible: true,
            color: None,
            outline: MFEKOutline::new(),
            operation: None,
            images: vec![],
        };

        self.history.add_undo_entry(HistoryEntry {
            description: "Added layer.".to_owned(),
            layer_idx: self.layer_idx,
            contour_idx: self.contour_idx,
            point_idx: self.point_idx,
            selected: Some(self.selected.clone()),
            layer: new_layer.clone(), // dummy
            kind: HistoryType::LayerAdded
        });

        self.glyph.as_mut().unwrap().layers.push(new_layer);
        
        self.end_layer_modification();

        self.layer_idx = Some(self.glyph.as_mut().unwrap().layers.len() - 1);
        self.contour_idx = None;
        self.point_idx = None;
        self.selected.clear();
        self.mark_preview_dirty();
    }

    /// Deletes a layer. Generates a history entry and sets the user's selection to the layer above.
    pub fn delete_layer(&mut self, idx: usize, add_history: bool) {
        if self.with_glyph(|glif| {glif.layers.len()}) == 1 { return }

        self.end_layer_modification();

        let deleted = self.glyph.as_mut().unwrap().layers.remove(idx);

        if add_history {
            self.history.add_undo_entry(HistoryEntry {
                description: "Deleted layer.".to_owned(),
                layer_idx: self.layer_idx,
                contour_idx: self.contour_idx,
                point_idx: self.point_idx,
                selected: Some(self.selected.clone()),
                layer: deleted.clone(), // dummy
                kind: HistoryType::LayerDeleted,
            });
        }

        if self.layer_idx != Some(0) {
            self.layer_idx = Some(self.layer_idx.unwrap() - 1);
        }
        self.contour_idx = None;
        self.point_idx = None;
        self.selected.clear();
        self.mark_preview_dirty();
    }

    pub fn set_active_layer(&mut self, idx: usize) {
        if let Some(old_idx) = self.layer_idx {
            if idx != old_idx {
                self.reset_tool();
            }
        }
        // TODO: save selection when leaving layer
        self.layer_idx = Some(idx);
        self.contour_idx = None;
        self.point_idx = None;
        self.selected.clear();
    }

    pub fn swap_active_layer(&mut self, idx: usize) {
        self.layer_idx = Some(idx);
    }

    pub fn get_active_layer(&self) -> usize {
        return self.layer_idx.unwrap();
    }
    
    pub fn swap_layers(&mut self, src: usize, dest: usize, add_history: bool) {
        if add_history {
            self.history.add_undo_entry(HistoryEntry {
                description: "Layer moved.".to_owned(),
                layer_idx: self.layer_idx,
                contour_idx: self.contour_idx,
                point_idx: self.point_idx,
                selected: Some(self.selected.clone()),
                layer: self.glyph.as_ref().unwrap().layers[self.layer_idx.unwrap()].clone(),
                kind: HistoryType::LayerMoved { to: dest, from: src }
            });
    
        }

        let src_copy = self.glyph.as_mut().unwrap().layers[src].clone();
        let dest_copy = self.glyph.as_mut().unwrap().layers[dest].clone();

        if self.layer_idx == Some(dest) { self.layer_idx = Some(src) };
        if self.layer_idx == Some(src) { self.layer_idx = Some(dest) };

        self.glyph.as_mut().unwrap().layers[dest] = src_copy;
        self.glyph.as_mut().unwrap().layers[src] = dest_copy;

        if dest == 0 && self.glyph.as_ref().unwrap().layers[src].operation.is_none() {
            self.glyph.as_mut().unwrap().layers[dest].operation = None;
        }

        self.mark_preview_dirty();
    }
    
    pub fn get_layer_count(&self) -> usize {
        return self.glyph.as_ref().unwrap().layers.len();
    }
}
