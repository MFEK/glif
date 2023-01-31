use std::rc::Rc;

use egui::{Context, Align2};
use glifparser::glif::LayerOperation;

use crate::editor::Editor;
use crate::user_interface::gui::{PROMPT_CLR};
use crate::user_interface::{icons, InputPrompt};
use crate::Interface;

pub fn layer_list(ctx: &Context, v: &mut Editor, i: &mut Interface) {
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

                        if ui.add(eye_button).clicked() {
                            let active_layer = v.get_active_layer();
                            v.set_active_layer(layer);
                
                            v.begin_modification("Toggled layer visibility.", false);
                            v.get_active_layer_mut().visible = !v.get_active_layer_ref().visible;
                            v.end_modification();
                
                            v.set_active_layer(active_layer);
                        }

                        if ui.button("📛").clicked() {
                            i.push_prompt(InputPrompt::Text {
                                label: "Layer name:".to_string(),
                                default: v.with_glyph(|glif| glif.layers[layer].name.clone()),
                                func: Rc::new(move |editor, string| {
                                    let active_layer = editor.get_active_layer();
                                    editor.set_active_layer(layer);
                
                                    editor.begin_modification("Renamed layer.", false);
                                    editor.get_active_layer_mut().name = string;
                                    editor.end_modification();
                
                                    editor.set_active_layer(active_layer);
                                }),
                            });
                        }

                        let popup_id = ui.make_persistent_id(format!("layer{0}popup", layer ));
                        let current_operation = v.with_glyph(|glif| glif.layers[layer].operation.clone());
                        let mut selected_operation = current_operation.clone();
                        let response = ui.button("💇");
                        let mut clicked = false;
                        if response.clicked() {
                            ui.memory().toggle_popup(popup_id);
                            clicked = true;
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
                            if ui.button("💧").clicked() {
                                    i.push_prompt(InputPrompt::Color {
                                        label: "Layer color:".to_string(),
                                        default: v.with_glyph(|glif| {
                                            let default = glif.layers[layer]
                                                .color
                                                .unwrap_or([0., 0., 0., 1.].into())
                                                .into();
                                            PROMPT_CLR.with(|clr| *clr.borrow_mut() = default);
                                            default
                                        }),
                                        func: Rc::new(move |editor, color| {
                                            let active_layer = editor.get_active_layer();
                                            editor.set_active_layer(layer);
                    
                                            editor.begin_modification("Changed layer color.", false);
                                            editor.get_active_layer_mut().color = color.map(|c| c.into());
                                            editor.end_modification();
                    
                                            editor.set_active_layer(active_layer);
                                        }),
                                    });
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
pub fn build_and_check_layer_list(v: &mut Editor, i: &mut Interface, ui: &egui::Ui) {
    /*
        ui.button(
            unsafe { imgui::ImStr::from_utf8_with_nul_unchecked(icon) },
            [0., 0.],
        );
        if ui.is_item_clicked(imgui::MouseButton::Right) {
            let active_layer = v.get_active_layer();
            v.set_active_layer(layer);
            v.begin_modification("Changed layer operation.");
            v.get_active_layer_mut().operation = None;
            v.end_modification();
            v.set_active_layer(active_layer);
        }
        if ui.is_item_clicked(imgui::MouseButton::Left) {
            let new_operation = match current_operation {
                Some(op) => match op {
                    LayerOperation::Difference => Some(LayerOperation::Union),
                    LayerOperation::Union => Some(LayerOperation::XOR),
                    LayerOperation::XOR => Some(LayerOperation::Intersect),
                    LayerOperation::Intersect => None,
                },
                None => Some(LayerOperation::Difference),
            };

            let active_layer = v.get_active_layer();
            v.set_active_layer(layer);
            v.begin_modification("Changed layer operation.");
            v.get_active_layer_mut().operation = new_operation;
            v.end_modification();
            v.set_active_layer(active_layer);
        }

        if layer_op.is_none() {
            ui.same_line(0.);
            let mut color_token: Option<ColorStackToken> = None;
            let _default_color: Option<[f32; 4]> = None;
            if let Some(color) = v.with_glyph(|glif| glif.layers[layer].color) {
                color_token = Some(ui.push_style_color(imgui::StyleColor::Button, color.into()));
            }
            ui.button(imgui::im_str!("##"), [0., 0.]);
            if ui.is_item_clicked(imgui::MouseButton::Left) {
                i.push_prompt(InputPrompt::Color {
                    label: "Layer color:".to_string(),
                    default: v.with_glyph(|glif| {
                        let default = glif.layers[layer]
                            .color
                            .unwrap_or([0., 0., 0., 1.].into())
                            .into();
                        PROMPT_CLR.with(|clr| *clr.borrow_mut() = default);
                        default
                    }),
                    func: Rc::new(move |editor, color| {
                        let active_layer = editor.get_active_layer();
                        editor.set_active_layer(layer);

                        editor.begin_modification("Changed layer color.");
                        editor.get_active_layer_mut().color = color.map(|c| c.into());
                        editor.end_modification();

                        editor.set_active_layer(active_layer);
                    }),
                });
            }

            if let Some(token) = color_token {
                token.pop(ui);
            }
        }

        font_token.pop(ui);
        custom_button_color.pop(ui);

        ui.same_line(0.);
        let mut pop_me = None;
        if active_layer != layer {
            pop_me = Some(ui.push_style_color(imgui::StyleColor::Button, [0., 0., 0., 0.2]));
        }
        ui.button(&im_str, [-1., 0.]);
        if ui.is_item_clicked(imgui::MouseButton::Left) {
            v.set_active_layer(layer);
        }
        if let Some(p) = pop_me {
            p.pop(ui);
        }
        no_padding.pop(ui);
    }
    */
}
