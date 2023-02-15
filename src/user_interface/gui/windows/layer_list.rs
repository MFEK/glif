use std::collections::HashMap;
use std::rc::Rc;

use egui::{Context, Align2, Stroke, Color32, PointerButton};
use glifparser::Color;
use glifparser::glif::LayerOperation;

use crate::editor::Editor;
use crate::user_interface::gui::textedit_buffer::EditBuffer;
use crate::user_interface::gui::{PROMPT_CLR};
use crate::user_interface::{icons, InputPrompt};
use crate::Interface;

pub struct LayerList {
    popup_had_focus: bool,
    cur_popup: Option<String>,
    edit_buf: HashMap<String, String>
}

impl LayerList {
    pub fn new() -> Self {
        Self {
            popup_had_focus: false,
            cur_popup: None,
            edit_buf: HashMap::new()
        }
    }
    pub fn build(&mut self, ctx: &Context, v: &mut Editor, i: &mut Interface) {
        let active_layer = v.get_active_layer();
    
        egui::Window::new("Layers")
            .resizable(true)
            .vscroll(true)
            .default_width(200.)
            .enabled(!v.is_modifying())
            .anchor(Align2::RIGHT_BOTTOM, [0., 0.])
            .resize(|r| {
                r.default_width(200.)
            })
            .show(ctx, |ui| {
                let mut selected_layer = v.get_active_layer();
                ui.horizontal(|ui| {
                    if ui.button("➕").clicked() {
                        v.new_layer();
                    }
    
                    if ui.button("➖").clicked() {
                        v.delete_layer();
                    }
    
                    if ui.button("⮫").clicked() {
                        v.swap_layers(active_layer, active_layer - 1, true);
                    }
    
                    
                    if ui.button("⮨").clicked() {
                        v.swap_layers(active_layer, active_layer + 1, true);
                    }
                });  
    
                ui.separator();
    
                let layer_count = v.get_layer_count();
                    for layer in 0..layer_count {
                        
                        ui.horizontal(|ui| {
                            // Show/hide button eye is visible shades is hidden
                            let eye_con = if v.with_glyph(|glif| glif.layers[layer].visible) {
                                "👁"
                            } else {
                                "👓"
                            };
    
                            let eye_button = egui::Button::new(eye_con)
                                .min_size(egui::vec2(24., 0.));

    
                            if ui.add(eye_button).on_hover_text("Visible").clicked() {
                                let active_layer = v.get_active_layer();
                                v.set_active_layer(layer);
                    
                                v.begin_modification("Toggled layer visibility.", false);
                                v.get_active_layer_mut().visible = !v.get_active_layer_ref().visible;
                                v.end_modification();
                    
                                v.set_active_layer(active_layer);
                            }
    
                            let response = ui.button("📛").on_hover_text("Rename");

                            let popup_id = ui.make_persistent_id(format!("layer{0}namepopup", layer ));
                            let name_clicked = response.clicked();
                            if response.clicked() {
                                self.cur_popup = Some(format!("layer{0}namepopup", layer ).to_string());
                            }

                            if self.cur_popup == Some(format!("layer{0}namepopup", layer ).to_string()) {
                                ui.memory().open_popup(popup_id);
                            }
    
                            egui::popup::popup_above_or_below_widget(ui, popup_id, &response, egui::AboveOrBelow::Above, |ui| {
                                ui.set_min_width(64.);
                                let name = v.with_glyph(|glif| glif.layers[layer].name.clone());
                                let edit_response = ui.text_edit_singleline(self.edit_buf.get_buf("name", &name));
                                if edit_response.lost_focus() && self.popup_had_focus {
                                    if self.edit_buf.get_buf("name", &name).clone() != name {
                                        let active_layer = v.get_active_layer();
                                        v.set_active_layer(layer);
                                        v.begin_modification("Changed layer name.", true);
                                        v.get_active_layer_mut().name = self.edit_buf.get_buf("name", &name).clone();
                                        v.end_modification();
                                        v.set_active_layer(active_layer)
                                    }

                                    self.popup_had_focus = false;
                                    self.cur_popup = None;
                                    ui.memory().close_popup();
                                }

                                if !edit_response.has_focus() {
                                    self.edit_buf.reset("name");
                                } else {
                                    self.popup_had_focus = true;
                                }

                                if edit_response.clicked_elsewhere() && !name_clicked {
                                    self.popup_had_focus = false;
                                    self.cur_popup = None;
                                    ui.memory().close_popup();
                                }
                            });
    
                            let popup_id = ui.make_persistent_id(format!("layer{0}popup", layer ));
                            let current_operation = v.with_glyph(|glif| glif.layers[layer].operation.clone());
                            let mut selected_operation = current_operation.clone();

                            let response = ui.button("💇").on_hover_text("Layer Operation");
                            if response.clicked() {
                                ui.memory().toggle_popup(popup_id);
                            }
    
                            egui::popup::popup_above_or_below_widget(ui, popup_id, &response, egui::AboveOrBelow::Above, |ui| {
                                ui.set_min_width(96.);
                                if ui.button("None").clicked() {
                                    selected_operation = None;
                                }
                                if ui.button("Difference").clicked() {
                                    selected_operation = Some(LayerOperation::Difference);
                                }
                                if ui.button("Union").clicked() {
                                    selected_operation = Some(LayerOperation::Union);
                                }
                                if ui.button("Xor").clicked() {
                                    selected_operation = Some(LayerOperation::XOR);
                                }
                                if ui.button("Intersect").clicked() {
                                    selected_operation = Some(LayerOperation::Intersect);
                                }
                            }); 
    
                            if selected_operation != current_operation{
                                let active_layer = v.get_active_layer();
                                v.set_active_layer(layer);
                                v.begin_modification("Changed layer operation.", false);
                                v.get_active_layer_mut().operation = selected_operation;
                                v.end_modification();
                                v.set_active_layer(active_layer);
                            }
    
                            if current_operation.is_none() {
                                let cur_color = v.with_glyph(|g| g.layers[layer].color);
                            
                                let mut color_array: [f32;4] = cur_color.unwrap_or(Color::default()).into();
                                let orig_color = color_array.clone();
                                ui.set_min_width(24.);
                                ui.color_edit_button_rgba_unmultiplied(&mut color_array);

                                if color_array != orig_color {
                                    let active_layer = v.get_active_layer(); 
                                    v.set_active_layer(layer);
            
                                    v.begin_modification("Changed layer color.", true);
                                    v.get_active_layer_mut().color = Some(color_array.into());
                                    v.end_modification();
            
                                    v.set_active_layer(active_layer);
                                }
                            }
                            
                            let layer_name = v.with_glyph(|g| g.layers[layer].name.clone());
                            ui.selectable_value(&mut selected_layer, layer, layer_name);
                        });
                    }
    
                
                if selected_layer != v.get_active_layer() {
                    v.set_active_layer(selected_layer);
                }
    
            });
    }
}