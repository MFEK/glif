use std::rc::Rc;

use super::super::prelude::*;
use super::Image;
use imgui::Ui;
use crate::user_interface::{InputPrompt, Interface};

fn imgui_decimal_text_field(label: &str, ui: &imgui::Ui, data: &mut f32) {
    let mut x = imgui::im_str!("{}", data);
    let label = imgui::ImString::new(label);
    let entered;
    {
    let it = ui.input_text(&label, &mut x);
    entered = it.enter_returns_true(true)
        .chars_decimal(true)
        .chars_noblank(true)
        .auto_select_all(true)
        .build();
    }
    if entered {
        if x.to_str().len() > 0 {
            let new_x: f32 = x.to_str().parse().unwrap();
            *data = new_x;
        }
    }
}

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
        .position(
            [tx, ty],
            imgui::Condition::Always,
        )
        .size(
            [tw, th],
            imgui::Condition::Always,
        )
        .build(ui, || {
            if let Some(selected) = self.selected_idx {
                ui.button(imgui::im_str!("Color"), [0., 0.]);
                if ui.is_item_clicked(imgui::MouseButton::Left) {
                    i.push_prompt(InputPrompt::Color {
                        label: "Layer color:".to_string(),
                        default: v.with_active_layer(|layer| layer.images[selected].0.color.unwrap_or([1., 1., 1., 1.].into())).into(),
                        func: Rc::new(move |editor, color| {            
                            editor.begin_layer_modification("Changed image color.");
                            editor.with_active_layer_mut(|layer| layer.images[selected].0.color = color.map(|c|c.into()));
                            editor.end_layer_modification();

                            editor.recache_images();
                        }),
                    });
                }
                ui.same_line(0.);

                ui.button(imgui::im_str!("Reset"), [0., 0.]);
                if ui.is_item_clicked(imgui::MouseButton::Left) {
                    let default_mat = kurbo::Affine::default();
                    v.begin_layer_modification("Reset image transform.");
                    v.with_active_layer_mut(|layer| layer.images[selected].1 = default_mat);
                    v.end_layer_modification();
                }
            }
        });
    }
}