use crate::editor::Editor;
use crate::user_interface::gui::{PROMPT_CLR, PROMPT_STR, TOOLBOX_HEIGHT, TOOLBOX_WIDTH};
use crate::user_interface::{InputPrompt, Interface};
use imgui::{Key, StyleVar};

pub fn build_and_check_prompts(v: &mut Editor, i: &mut Interface, ui: &mut imgui::Ui) {
    if !i.active_prompts() {
        return;
    };

    imgui::Window::new(&ui, format!("##"))
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
        .build(|| {
            ui.invisible_button(format!("##"), [-1., -1.]);
            if ui.is_item_clicked_with_button(imgui::MouseButton::Right) {
                i.pop_prompt();
            }
        });

    match i.peek_prompt().clone() {
        InputPrompt::Text {
            label,
            default: _,
            func,
        } => {
            imgui::Window::new(&ui, format!("{}", label))
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
                .build(|| {
                    PROMPT_STR.with(|prompt_str| {
                        let _tok = ui.push_item_width(-1.);
                        prompt_str.borrow_mut().clear();
                        ui.input_text("", &mut prompt_str.borrow_mut())
                            .build();

                        if ui.is_key_down(Key::Enter) {
                            let final_string = prompt_str.borrow().to_string();
                            let mut new_string = String::new();
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

            imgui::Window::new(&ui, format!("{}", label))
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
                .build(|| {
                    PROMPT_CLR.with(|ui_color| {
                        imgui::ColorPicker4::new(format!("{}", label), &mut color).build(ui);

                        if ui.is_key_down(Key::Enter) {
                            ui_color.replace([0., 0., 0., 1.]);
                            func(v, Some(color));
                            i.pop_prompt();
                        }

                        ui.button("Automatic");
                        if ui.is_item_clicked() {
                            func(v, None);
                            i.pop_prompt();
                        }
                    })
                });

            PROMPT_CLR.with(|clr| clr.replace(color));
        }

        InputPrompt::Layer { label, func } => {
            imgui::Window::new(&ui, format!("{}", label))
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
                .build(|| {
                    let layer_count = v.with_glyph(|glif| glif.layers.len());
                    for layer in 0..layer_count {
                        let layer_op = v.with_glyph(|glif| glif.layers[layer].operation.clone());
                        let im_str = format!(
                            "{0}",
                            v.with_glyph(|glif| { glif.layers[layer].name.clone() })
                        );

                        let no_padding = ui.push_style_var(StyleVar::ItemSpacing([0., 0.]));

                        if layer_op.is_some() {
                            ui.dummy([28., 0.]);
                            ui.same_line();
                        }

                        let mut pop_me = None;
                        if v.get_active_layer() != layer {
                            pop_me = Some(
                                ui.push_style_color(imgui::StyleColor::Button, [0., 0., 0., 0.2]),
                            );
                        }
                        ui.button(&im_str);
                        if ui.is_item_clicked() {
                            func(v, v.with_glyph(|glif| glif.layers[layer].clone()));
                            i.pop_prompt();
                        }
                        if let Some(p) = pop_me {
                            p.pop();
                        }
                        no_padding.pop();
                    }
                });
        }
    }
}
