use std::rc::Rc;

use super::super::prelude::*;
use super::Image;
use crate::user_interface::{InputPrompt, Interface, gui::windows::egui_parsed_textfield};
use egui::Ui;
use glifparser::IntegerOrFloat;

impl Image {
    pub fn tool_dialog(&mut self, v: &mut Editor, i: &mut Interface, ui: &mut Ui) {
        if let Some(selected) = self.selected_idx {
            let mut image = v.get_active_layer_ref().images[selected].clone();
            ui.label("Position");
            image.0.xOffset = IntegerOrFloat::Float(egui_parsed_textfield(ui, "px", image.0.xOffset.into(), &mut self.edit_buf));
            image.0.yOffset = IntegerOrFloat::Float(egui_parsed_textfield(ui, "py", image.0.xOffset.into(), &mut self.edit_buf));
            let o_rgba: [f32; 4] =  v.get_active_layer_mut().images[selected].0.color.unwrap_or_else(|| [1., 1., 1., 1.].into()).into();
            let mut rgba = o_rgba.clone();
            if ui.color_edit_button_rgba_unmultiplied(&mut rgba).changed() {
                v.begin_modification("Changed image color.", true);
                v.get_active_layer_mut().images[selected].0.color = Some(rgba.into());
                v.end_modification();

                v.recache_images();    
            }

            if image != v.get_active_layer_ref().images[selected].clone() {
                v.begin_modification("Edited image with image window.", false);
                v.get_active_layer_mut().images[selected] = image;
                v.end_modification()
            }
        }
        /*
                if let Some(selected) = self.selected_idx {
                    ui.button(imgui::im_str!("Color"), [0., 0.]);
                    if ui.is_item_clicked(imgui::MouseButton::Left) {
                        i.push_prompt(InputPrompt::Color {
                            label: "Layer color:".to_string(),
                            default: v.get_active_layer_mut().images[selected]
                                .0
                                .color
                                .unwrap_or_else(|| [1., 1., 1., 1.].into())
                                .into(),
                            func: Rc::new(move |editor, color| {
                                editor.begin_modification("Changed image color.");
                                editor.get_active_layer_mut().images[selected].0.color =
                                    color.map(|c| c.into());
                                editor.end_modification();

                                editor.recache_images();
                            }),
                        });
                    }
                    ui.same_line(0.);

                    ui.button(imgui::im_str!("Reset"), [0., 0.]);
                    if ui.is_item_clicked(imgui::MouseButton::Left) {
                        let default_mat = kurbo::Affine::default();
                        v.begin_modification("Reset image transform.");
                        v.get_active_layer_mut().images[selected].1 = default_mat;
                        v.end_modification();
                    }
                }
            });
        */
    }
}
