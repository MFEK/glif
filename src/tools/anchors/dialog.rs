use crate::{user_interface::{gui::windows::egui_parsed_textfield, Interface}, editor::Editor};

use super::Anchors;


// Make dialog box at right
impl Anchors {
    pub fn anchor_settings(&mut self, v: &mut Editor, _: &Interface, ui: &mut egui::Ui) {
        let anchor_idx = self.anchor_idx.expect("We check in the event handler!");
        let mut anchor = v.with_glyph(|glif| glif.anchors[anchor_idx].clone());

        ui.label("Position");
        anchor.x = egui_parsed_textfield(ui, "ax", anchor.x, &mut self.edit_buf);
        anchor.y = egui_parsed_textfield(ui, "ay", anchor.y, &mut self.edit_buf);

        let mut class = anchor.class.as_ref().unwrap_or(&String::new()).clone();
        ui.text_edit_singleline(&mut class);

        if !class.is_empty() {
            anchor.class = Some(class);
        } else {
            anchor.class = None;
        }

        if anchor != v.with_glyph(|glif| glif.anchors[anchor_idx].clone()) {
            v.with_glyph_mut(|glif| {
                glif.anchors[anchor_idx] = anchor.clone();
            });
        }

    }
}