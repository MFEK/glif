use crate::editor::Editor;
use crate::user_interface::gui::{PROMPT_CLR, PROMPT_STR, TOOLBOX_HEIGHT, TOOLBOX_WIDTH};
use crate::user_interface::{InputPrompt, Interface};
use imgui::{Key, StyleVar};

pub fn build_and_check_prompts(v: &mut Editor, i: &mut Interface, ui: &mut imgui::Ui) {
    if !i.active_prompts() {
        return;
    };

    imgui::Window::new(&imgui::im_str!("##"))
        .flags(
            imgui::WindowFlags::NO_RESIZE
                | imgui::WindowFlags::NO_MOVE
                | imgui::WindowFlags::NO_COLLAPSE
                | imgui::WindowFlags::NO_DECORATION
                | imgui::WindowFlags::NO_BACKGROUND,
        )
        .position([0., 0.], imgui::Condition::Always)
        .size(
            [i.viewport.winsize.0 as f32, i.viewport.winsize.1 as f32],
            imgui::Condition::Always,
        )
        .build(ui, || {
            ui.invisible_button(&imgui::im_str!("##"), [-1., -1.]);
            if ui.is_item_clicked(imgui::MouseButton::Right) {
                i.pop_prompt();
            }
        });

    match i.peek_prompt().clone() {
        InputPrompt::Text {
            label,
            default: _,
            func,
        } => {
            imgui::Window::new(&imgui::im_str!("{}", label))
                .bg_alpha(1.) // See comment on fn redraw_skia
                .flags(imgui::WindowFlags::NO_RESIZE | imgui::WindowFlags::NO_COLLAPSE)
                .position_pivot([0.5, 0.5])
                .position(
                    [
                        (i.viewport.winsize.0 / 2.),
                        (i.viewport.winsize.1 / 2.),
                    ],
                    imgui::Condition::Always,
                )
                .size(
                    [*TOOLBOX_HEIGHT, TOOLBOX_WIDTH + 10.],
                    imgui::Condition::Always,
                )
                .focused(true)
                .build(ui, || {
                    PROMPT_STR.with(|prompt_str| {
                        ui.push_item_width(-1.);
                        prompt_str.borrow_mut().clear();
                        ui.input_text(imgui::im_str!(""), &mut prompt_str.borrow_mut())
                            .build();

                        if ui.is_key_down(Key::Enter) {
                            let final_string = prompt_str.borrow().to_string();
                            let mut new_string = imgui::ImString::new("");
                            new_string.reserve(256);
                            prompt_str.replace(new_string);
                            func(v, final_string);
                            i.pop_prompt();
                        }
                    })
                });
        }

        InputPrompt::Color {
            label,
            default: _,
            func,
        } => {
            let mut color = PROMPT_CLR.with(|clr| clr.borrow_mut().clone());

            imgui::Window::new(&imgui::im_str!("{}", label))
                .bg_alpha(1.) // See comment on fn redraw_skia
                .flags(imgui::WindowFlags::NO_RESIZE | imgui::WindowFlags::NO_COLLAPSE)
                .position_pivot([0.5, 0.5])
                .position(
                    [
                        (i.viewport.winsize.0 / 2.),
                        (i.viewport.winsize.1 / 2.),
                    ],
                    imgui::Condition::Always,
                )
                .size(
                    [*TOOLBOX_HEIGHT, *TOOLBOX_HEIGHT + 10.],
                    imgui::Condition::Always,
                )
                .focused(true)
                .build(ui, || {
                    PROMPT_CLR.with(|ui_color| {
                        imgui::ColorPicker::new(&imgui::im_str!("{}", label), &mut color).build(ui);

                        if ui.is_key_down(Key::Enter) {
                            ui_color.replace([0., 0., 0., 1.]);
                            func(v, Some(color));
                            i.pop_prompt();
                        }

                        ui.button(imgui::im_str!("Automatic"), [0., 0.]);
                        if ui.is_item_clicked(imgui::MouseButton::Left) {
                            func(v, None);
                            i.pop_prompt();
                        }
                    })
                });

            PROMPT_CLR.with(|clr| clr.replace(color));
        }

        InputPrompt::Layer { label, func } => {
            imgui::Window::new(&imgui::im_str!("{}", label))
                .bg_alpha(1.) // See comment on fn redraw_skia
                .flags(imgui::WindowFlags::NO_RESIZE | imgui::WindowFlags::NO_COLLAPSE)
                .position_pivot([0.5, 0.5])
                .position(
                    [
                        (i.viewport.winsize.0 / 2.),
                        (i.viewport.winsize.1 / 2.),
                    ],
                    imgui::Condition::Always,
                )
                .size(
                    [*TOOLBOX_HEIGHT, *TOOLBOX_HEIGHT + 10.],
                    imgui::Condition::Always,
                )
                .focused(true)
                .build(ui, || {
                    let layer_count = v.with_glyph(|glif| glif.layers.len());
                    for layer in 0..layer_count {
                        let layer_op = v.with_glyph(|glif| glif.layers[layer].operation.clone());
                        let layer_temp_name = imgui::im_str!(
                            "{0}",
                            v.with_glyph(|glif| { glif.layers[layer].name.clone() })
                        );
                        let im_str = imgui::ImString::from(layer_temp_name);

                        let no_padding = ui.push_style_var(StyleVar::ItemSpacing([0., 0.]));

                        if layer_op.is_some() {
                            ui.dummy([28., 0.]);
                            ui.same_line(0.);
                        }

                        let mut pop_me = None;
                        if v.get_active_layer() != layer {
                            pop_me = Some(
                                ui.push_style_color(imgui::StyleColor::Button, [0., 0., 0., 0.2]),
                            );
                        }
                        ui.button(&im_str, [-1., 0.]);
                        if ui.is_item_clicked(imgui::MouseButton::Left) {
                            func(v, v.with_glyph(|glif| glif.layers[layer].clone()));
                            i.pop_prompt();
                        }
                        if let Some(p) = pop_me {
                            p.pop(ui);
                        }
                        no_padding.pop(ui);
                    }
                });
        }
    }
}
