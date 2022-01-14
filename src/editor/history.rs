use glifparser::glif::HistoryEntry;

use super::Editor;
use crate::util::MFEKGlifPointData;

use std::mem;

#[derive(Clone, Debug, Default)]
pub struct History<PD: glifparser::PointData> {
    pub undo_stack: Vec<HistoryEntry<PD>>,
    pub redo_stack: Vec<HistoryEntry<PD>>,
}

impl<PD: glifparser::PointData> History<PD> {
    pub fn add_undo_entry(&mut self, entry: HistoryEntry<PD>) {
        log::debug!("Added undo entry: {0}", entry.description);
        self.undo_stack.push(entry);
        self.redo_stack.clear();
    }
}

pub fn entry_from_desc_and_editor(desc: &str, v: &Editor) -> HistoryEntry<MFEKGlifPointData> {
    HistoryEntry {
        description: desc.to_owned(),
        layer_idx: v.layer_idx,
        contour_idx: v.contour_idx,
        point_idx: v.point_idx,
        guidelines: v.guidelines.clone(),
        selected: Some(v.selected.clone()),
        glyph: v.glyph.as_ref().unwrap().clone(),
    }
}

impl Editor {
    /// Pops a HistoryEntry off the layer stack and restores it.
    pub fn undo(&mut self) {
        if self.modifying {
            log::trace!("Tried to undo while modifying — dropped.");
            return;
        }
        let entry = self.history.undo_stack.pop();
        log::trace!("Popped undo stack: {:?}", &entry);

        if let Some(undo_entry) = entry {
            log::debug!("Undid {}", &undo_entry.description);
            self.history
                .redo_stack
                .push(entry_from_desc_and_editor("Undo", self));

            self.glyph = Some(undo_entry.glyph.clone());
            self.layer_idx = undo_entry.layer_idx;
            self.contour_idx = undo_entry.contour_idx;
            self.point_idx = undo_entry.point_idx;
            self.guidelines = undo_entry.guidelines;
            if let Some(selected) = undo_entry.selected {
                self.selected = selected
            }

            self.mark_preview_dirty();
        }
    }

    pub fn redo(&mut self) {
        if self.modifying {
            return;
        }
        let entry = self.history.redo_stack.pop();
        log::trace!("Popped redo stack: {:?}", &entry);

        if let Some(redo_entry) = entry {
            log::debug!("Redid {}", &redo_entry.description);
            self.history.undo_stack.push(HistoryEntry {
                description: "Redo".to_owned(),
                layer_idx: self.layer_idx,
                contour_idx: self.contour_idx,
                point_idx: self.point_idx,
                guidelines: self.guidelines.clone(),
                selected: Some(self.selected.clone()),
                glyph: self.glyph.as_ref().unwrap().clone(),
            });

            self.glyph = Some(redo_entry.glyph.clone());
            self.layer_idx = redo_entry.layer_idx;
            self.contour_idx = redo_entry.contour_idx;
            self.point_idx = redo_entry.point_idx;
            self.guidelines = redo_entry.guidelines;
            if let Some(selected) = redo_entry.selected {
                self.selected = selected
            }

            self.mark_preview_dirty();
        }
    }

    /// This function combines entries on the top of the undo stack that share a description.
    pub fn collapse_history_entries(&mut self) {
        let top_entry = self.history.undo_stack.pop();

        if let Some(entry) = top_entry {
            loop {
                let next_entry = self.history.undo_stack.pop();
                if let Some(next_entry) = next_entry {
                    if next_entry.description != entry.description {
                        self.history.undo_stack.push(next_entry);
                        break;
                    }
                } else {
                    break;
                }
            }

            self.history.undo_stack.push(entry);
        }
    }

    /// Sometimes users might do things while a modification is in progress that fundamentally
    /// change its nature to the degree the undo label originally bestowed no longer makes sense.
    /// Use of this function should be done with care, but not avoided for convenience sake.
    pub fn redescribe_modification(&mut self, mut description: String) {
        if !self.modifying {
            // History integrity in doubt!
            log::error!("Tried to redescribe a modification when not modifying!");
        } else {
            match self.history.undo_stack.last_mut().as_mut() {
                Some(he) => {
                    if he.description == description {
                        log::trace!("redescribe_modification(…): Requested history redescribe is a no-op…bailing");
                    } else {
                        mem::swap(&mut he.description, &mut description);
                        log::debug!("Undo formerly “{}” now “{}”", &description, &he.description);
                    }
                }
                None => log::error!("Nothing on undo stack — failed to swap!?"),
            }
        }
    }
}
