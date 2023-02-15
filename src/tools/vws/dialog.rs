use glifparser::CapType;
use glifparser::JoinType;
use glifparser::VWSContour;
use glifparser::glif::contour_operations::ContourOperations;

use super::super::prelude::*;
use super::util::*;
use super::VWS;
use crate::user_interface::Interface;
use crate::user_interface::gui::windows::egui_parsed_textfield;

impl VWS {
    pub fn tool_dialog(&mut self, v: &mut Editor, i: &Interface, ui: &mut egui::Ui) {
        if v.contour_idx.is_none() {
            ui.label("No selection!");
            return;
        }
        
        let (cidx, pidx) = v.selected_point().expect("Checked in function return gaurd.");
        let operation = v.get_active_layer_ref().outline[cidx].operation().clone();
        if let Some(ContourOperations::VariableWidthStroke { mut data }) = operation {
            let original_data = data.clone();

            ui.collapsing("Contour", |ui| {
                ui.label("Cap Types");
    
                fn cap_combo(ui: &mut egui::Ui, label: &str, cur_selection: &mut CapType) {
                    egui::ComboBox::from_label(label)
                        .selected_text(format!("{:?}", cur_selection))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(cur_selection, CapType::Round, "Round");
                            ui.selectable_value(cur_selection, CapType::Circle, "Circle");
                            ui.selectable_value(cur_selection, CapType::Square, "Squared");
                            ui.selectable_value(cur_selection, CapType::Custom, "Custom");
                        });
                }
                
                cap_combo(ui, "Start", &mut data.cap_start_type);
                cap_combo(ui, "End", &mut data.cap_end_type);
    
                ui.separator();
    
                egui::ComboBox::from_label("Join Type")
                    .selected_text(format!("{:?}", &mut data.join_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut data.join_type, JoinType::Round, "Round");
                        ui.selectable_value(&mut data.join_type, JoinType::Circle, "Circle");
                        ui.selectable_value(&mut data.join_type, JoinType::Miter, "Miter");
                        ui.selectable_value(&mut data.join_type, JoinType::Bevel, "Bevel");
                    });
    
                let mut max_offset: f64 = 0.;
                for handle in &data.handles {
                    max_offset = max_offset.max(handle.left_offset).max(handle.right_offset);
                }

                let mut offset = max_offset;
                ui.add(egui::Slider::new(&mut offset, 0.0..=50.));
    
                if offset != max_offset {
                    for handle in &mut data.handles {
                        handle.left_offset = offset;
                        handle.right_offset = offset;
                    }
                }
            });

            ui.collapsing("Point", |ui| {
                egui_parsed_textfield(ui, "lo", data.handles[pidx].left_offset, &mut self.edit_buf);
                egui_parsed_textfield(ui, "ro", data.handles[pidx].right_offset, &mut self.edit_buf);
            });

            if data != original_data {
                v.begin_modification("VWS dialog modification.", true);
                set_vws_contour(v, cidx, data);
                v.end_modification();
            }
        } else {
            ui.label("Non-VWS contour selected!");
        }
    }
}
