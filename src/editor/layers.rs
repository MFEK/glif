use std::collections::HashMap;

use glifparser::{Outline, glif::{HistoryEntry, HistoryType, Layer}};
use super::Editor;

impl Editor {
    /// Adds a new layer. This generates a HistoryEntry and sets the selection to point to the newly created
    /// layer.
    pub fn new_layer(&mut self) {
        let new_layer = Layer {
            outline: Some(Outline::new()),
            contour_ops: HashMap::new(),
        };

        self.history.push(HistoryEntry {
            description: "Added layer.".to_owned(),
            layer_idx: self.layer_idx,
            contour_idx: self.contour_idx,
            point_idx: self.point_idx,
            selected: Some(self.selected.clone()),
            layer: new_layer.clone(), // dummy
            kind: HistoryType::LayerAdded
        });

        self.glyph.as_mut().unwrap().glif.layers.push(new_layer);
        
        self.end_layer_modification();

        self.layer_idx = Some(self.glyph.as_mut().unwrap().glif.layers.len() - 1);
        self.contour_idx = None;
        self.point_idx = None;
        self.selected.clear();
    }

    /// Deletes a layer. Generates a history entry and sets the user's selection to the layer above.
    pub fn delete_layer(&mut self, idx: usize) {
        if self.with_glif(|glif| {glif.layers.len()}) == 1 { return }

        self.end_layer_modification();

        let deleted = self.glyph.as_mut().unwrap().glif.layers.remove(idx);
        self.history.push(HistoryEntry {
            description: "Deleted layer.".to_owned(),
            layer_idx: self.layer_idx,
            contour_idx: self.contour_idx,
            point_idx: self.point_idx,
            selected: Some(self.selected.clone()),
            layer: deleted.clone(), // dummy
            kind: HistoryType::LayerDeleted,
        });


        if self.layer_idx != Some(0) {
            self.layer_idx = Some(self.layer_idx.unwrap() - 1);
        }
        self.contour_idx = None;
        self.point_idx = None;
        self.selected.clear();
    }

    pub fn set_active_layer(&mut self, idx: usize) {
        // TODO: save selection when leaving layer
        self.layer_idx = Some(idx);
        self.contour_idx = None;
        self.point_idx = None;
        self.selected.clear();
    }

    pub fn get_active_layer(&self) -> usize {
        return self.layer_idx.unwrap();
    }

    
    pub fn swap_layers(&mut self, destination: usize) {
    }

    
    pub fn get_layer_count(&self) -> usize {
        return self.glyph.as_ref().unwrap().glif.layers.len();
    }
    
}