use std::rc::Rc;

use super::super::prelude::*;
use super::Image;
use crate::user_interface::{InputPrompt, Interface};
use imgui::Ui;

impl Image {
    pub fn tool_dialog(&mut self, v: &mut Editor, i: &mut Interface, ui: &Ui) {
        let (tx, ty, tw, th) = i.get_tools_dialog_rect();

        imgui::Window::new(&imgui::ImString::new("Image"))
            .bg_alpha(1.) // See comment on fn redraw_skia
            .flags(
                imgui::WindowFlags::NO_RESIZE
                    | imgui::WindowFlags::NO_MOVE
                    | imgui::WindowFlags::NO_COLLAPSE,
            )
            .position([tx, ty], imgui::Condition::Always)
            .size([tw, th], imgui::Condition::Always)
            .build(ui, || {
                if let Some(selected) = self.selected_idx {
                    ui.button(imgui::im_str!("Color"), [0., 0.]);
                    if ui.is_item_clicked(imgui::MouseButton::Left) {
                        i.push_prompt(InputPrompt::Color {
                            label: "Layer color:".to_string(),
                            default: v
                                .with_active_layer(|layer| {
                                    layer.images[selected]
                                        .0
                                        .color
                                        .unwrap_or_else(|| [1., 1., 1., 1.].into())
                                })
                                .into(),
                            func: Rc::new(move |editor, color| {
                                editor.begin_modification("Changed image color.");
                                editor.with_active_layer_mut(|layer| {
                                    layer.images[selected].0.color = color.map(|c| c.into())
                                });
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
                        v.with_active_layer_mut(|layer| layer.images[selected].1 = default_mat);
                        v.end_modification();
                    }
                }
            });
    }
}
