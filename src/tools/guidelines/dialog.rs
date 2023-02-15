use super::super::prelude::*;
use super::{Guidelines, SplitGuidelines};
use crate::tool_behaviors::add_guideline::AddGuideline;
//use crate::user_interface::gui::{FONT_IDS, IMGUI_RESERVE};
use crate::user_interface::icons;

lazy_static::lazy_static! {
    static ref PLUS_GLOBE: Vec<u8> = icons::chain(&[icons::PLUS, icons::GLOBE]);
}

impl Guidelines {
    pub fn tool_dialog(&mut self, v: &mut Editor, i: &mut Interface, ui: &mut egui::Ui) {
        let (guidelines, _, local_guidelines_len, _) = SplitGuidelines::new(v).as_tuple();

        ui.horizontal(|ui| {
            if ui.button("➕").clicked() {
                v.push_behavior(Box::new(AddGuideline::new(0., false)));
            }

            if ui.button("🌏").clicked() {
                v.push_behavior(Box::new(AddGuideline::new(0., true)));
            }

            if ui.button("🌐").clicked() {
                v.write_metrics(i);
            }
            
            if let Some(selected) = self.selected_idx {
                //log::trace!("Selected {}; global len {}", selected, global_guidelines_len);
                let selected_i = if guidelines[selected].1 {selected - local_guidelines_len} else {selected};
                //log::trace!("Selected index {}", selected_i);

                if guidelines[selected].1 && guidelines[selected].0.data.as_guideline().format {
                    ui.label("Format defined.");
                } else {
                    if ui.button("➖").clicked() {
                        v.begin_modification(&format!("Remove {} guideline.", if guidelines[selected].1 { "global" } else { "local" }), false);
                        self.selected_idx = None;
                        if guidelines[selected].1 {
                            v.guidelines.remove(selected_i);
                        } else {
                            v.with_glyph_mut(|glyph| {
                                glyph.guidelines.remove(selected_i);
                            });
                        }
                        v.end_modification();
                    }
                }
            } else {
                ui.label("No guideline selected.");
            }
        });
    }
}
