use super::super::prelude::*;
use super::{Guidelines, SplitGuidelines};
use crate::tool_behaviors::add_guideline::AddGuideline;
use crate::user_interface::gui::windows::egui_parsed_textfield;

impl Guidelines {
    pub fn tool_dialog(&mut self, v: &mut Editor, i: &mut Interface, ui: &mut egui::Ui) {
        let (mut guidelines, _, local_guidelines_len, _) = SplitGuidelines::new(v).as_tuple();

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
        if let Some(selected) = self.selected_idx {
            ui.label("Position");
            ui.vertical(|ui| {
                if guidelines[selected].1 {
                    /*v.begin_modification("Move guideline.", false);
                    v.with_glyph_mut(|glyph| {
                        let mut guidelinex = &mut glyph.guidelines[selected].at.x;
                        *guidelinex = egui_parsed_textfield(ui, "ax", *guidelinex, &mut self.edit_buf);
                        let mut guideliney = &mut glyph.guidelines[selected].at.y;
                        *guideliney = egui_parsed_textfield(ui, "ay", *guideliney, &mut self.edit_buf);
                    });
                    v.end_modification();*/
                } else {
                    let mut guidelinex = &mut v.guidelines[selected].at.x;
                    *guidelinex = egui_parsed_textfield(ui, "ax", *guidelinex, &mut self.edit_buf);
                    let mut guideliney = &mut v.guidelines[selected].at.y;
                    *guideliney = egui_parsed_textfield(ui, "ay", *guideliney, &mut self.edit_buf);
                }
            });
        }
    }
}
