use std::rc::Rc;

use glifparser::glif::LayerOperation;
use imgui::{ColorStackToken, StyleColor, StyleVar};

use crate::editor::Editor;
use crate::user_interface::gui::{FONT_IDS, PROMPT_CLR};
use crate::user_interface::{icons, InputPrompt};
use crate::Interface;

pub fn build_and_check_layer_list(v: &mut Editor, i: &mut Interface, ui: &imgui::Ui) {
    let active_layer = v.get_active_layer();

    let pop_me = ui.push_style_color(imgui::StyleColor::Button, [0., 0., 0., 0.2]);

    ui.button(
        unsafe { imgui::ImStr::from_utf8_with_nul_unchecked(icons::PLUS) },
        [0., 0.],
    );
    //ui.push_item_width(-0.5);
    if ui.is_item_clicked(imgui::MouseButton::Left) {
        v.new_layer();
    }

    ui.same_line(0.);
    ui.button(
        unsafe { imgui::ImStr::from_utf8_with_nul_unchecked(icons::MINUS) },
        [0., 0.],
    );
    ui.push_item_width(-0.5);
    if ui.is_item_clicked(imgui::MouseButton::Left) {
        v.delete_layer();
    }

    ui.same_line(0.);
    ui.button(
        unsafe { imgui::ImStr::from_utf8_with_nul_unchecked(icons::ARROWUP) },
        [0., 0.],
    );
    if ui.is_item_clicked(imgui::MouseButton::Left) {
        if active_layer != 0 {
            let _start_layer_type =
                v.with_glyph(|glif| glif.layers[active_layer].operation.clone());
            let _target_layer_type =
                v.with_glyph(|glif| glif.layers[active_layer - 1].operation.clone());

            v.swap_layers(active_layer, active_layer - 1, true);
        }
    }

    let layer_count = v.get_layer_count();
    ui.same_line(0.);
    ui.button(
        unsafe { imgui::ImStr::from_utf8_with_nul_unchecked(icons::ARROWDOWN) },
        [0., 0.],
    );
    if ui.is_item_clicked(imgui::MouseButton::Left) {
        if active_layer != layer_count - 1 {
            v.swap_layers(active_layer, active_layer + 1, true);
        }
    }

    pop_me.pop(ui);

    ui.separator();

    for layer in 0..layer_count {
        let layer_op = v.with_glyph(|glif| glif.layers[layer].operation.clone());
        let layer_temp_name = imgui::im_str!(
            "{0}",
            v.with_glyph(|glif| { glif.layers[layer].name.clone() })
        );
        let im_str = layer_temp_name;

        let font_token = ui.push_font(FONT_IDS.with(|ids| ids.borrow()[1]));
        let no_padding = ui.push_style_var(StyleVar::ItemSpacing([0., 0.]));
        let custom_button_color = ui.push_style_color(
            imgui::StyleColor::Button,
            ui.style_color(StyleColor::WindowBg),
        );

        if layer_op.is_some() {
            ui.dummy([28., 0.]);
            ui.same_line(0.);
        }
        let layer_visible = v.with_glyph(|glif| glif.layers[layer].visible);
        let eye_con = if layer_visible {
            icons::OPENEYE
        } else {
            icons::CLOSEDEYE
        };
        ui.button(
            unsafe { imgui::ImStr::from_utf8_with_nul_unchecked(eye_con) },
            [0., 0.],
        );
        if ui.is_item_clicked(imgui::MouseButton::Left) {
            let active_layer = v.get_active_layer();
            v.set_active_layer(layer);

            v.begin_modification("Toggled layer visibility.");
            v.get_active_layer_mut().visible = !v.get_active_layer_ref().visible;
            v.end_modification();

            v.set_active_layer(active_layer);
        }

        ui.same_line(0.);
        ui.button(
            unsafe { imgui::ImStr::from_utf8_with_nul_unchecked(icons::RENAME) },
            [0., 0.],
        );
        if ui.is_item_clicked(imgui::MouseButton::Left) {
            i.push_prompt(InputPrompt::Text {
                label: "Layer name:".to_string(),
                default: v.with_glyph(|glif| glif.layers[layer].name.clone()),
                func: Rc::new(move |editor, string| {
                    let active_layer = editor.get_active_layer();
                    editor.set_active_layer(layer);

                    editor.begin_modification("Renamed layer.");
                    editor.get_active_layer_mut().name = string;
                    editor.end_modification();

                    editor.set_active_layer(active_layer);
                }),
            });
        }
        ui.same_line(0.);

        let current_operation = v.with_glyph(|glif| glif.layers[layer].operation.clone());
        let icon = match current_operation.as_ref() {
            Some(op) => match op {
                LayerOperation::Difference => icons::LAYERDIFFERENCE,
                LayerOperation::Union => icons::LAYERUNION,
                LayerOperation::XOR => icons::LAYERXOR,
                LayerOperation::Intersect => icons::LAYERINTERSECTION,
            },
            None => icons::LAYERCOMBINE,
        };
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
}
