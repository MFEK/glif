use super::super::prelude::*;
use super::Guidelines;
use crate::tool_behaviors::add_guideline::AddGuideline;
use crate::user_interface::gui::FONT_IDS;
use crate::user_interface::{icons, InputPrompt};
use glifparser::IntegerOrFloat;
use imgui::StyleColor;
use std::rc::Rc;
use std::str as stdstr;

fn imgui_decimal_text_field(label: &str, ui: &imgui::Ui, data: &mut f32) {
    let mut x = format!("{}", data);
    let label = String::from(label);
    let entered;
    {
        let it = ui.input_text(&label, &mut x);
        entered = it
            .enter_returns_true(true)
            .chars_decimal(true)
            .chars_noblank(true)
            .auto_select_all(true)
            .build();
    }
    if entered {
        if x.as_str().len() > 0 {
            let new_x: f32 = x.as_str().parse().unwrap();
            *data = new_x;
        }
    }
}

impl Guidelines {
    pub fn tool_dialog(&mut self, v: &mut Editor, i: &mut Interface, ui: &imgui::Ui) {
        let (tx, ty, tw, th) = i.get_tools_dialog_rect();

        imgui::Window::new(&ui, &imgui::ImString::new("Guidelines"))
            .bg_alpha(1.) // See comment on fn redraw_skia
            .flags(
                imgui::WindowFlags::NO_RESIZE
                    | imgui::WindowFlags::NO_MOVE
                    | imgui::WindowFlags::NO_COLLAPSE,
            )
            .position([tx, ty], imgui::Condition::Always)
            .size([tw, th], imgui::Condition::Always)
            .build(|| {
                let pop_me = ui.push_style_color(imgui::StyleColor::Button, [0., 0., 0., 0.2]);

                ui.button(
                    unsafe { stdstr::from_utf8_unchecked(icons::PLUS) },
                );
                if ui.is_item_clicked() {
                    v.push_behavior(Box::new(AddGuideline::new()));
                }

                if let Some(selected) = self.selected_idx {
                    ui.same_line();
                    ui.button(
                        unsafe { stdstr::from_utf8_unchecked(icons::MINUS) },
                    );
                    if ui.is_item_clicked() {
                        v.with_glyph_mut(|glyph| {
                            self.selected_idx = None;
                            glyph.guidelines.remove(selected);
                        });
                    }
                }

                pop_me.pop();

                ui.separator();

                let guideline_count = v.with_glyph(|glif| glif.guidelines.len());

                for guideline in 0..guideline_count {
                    let im_str = v.with_glyph(|glif| {
                        { glif.guidelines[guideline].name.clone() }.unwrap_or("Unnamed".to_string())
                    });

                    let font_token = ui.push_font(FONT_IDS.with(|ids| ids.borrow()[1]));
                    let custom_button_color = ui.push_style_color(
                        imgui::StyleColor::Button,
                        ui.style_color(StyleColor::WindowBg),
                    );

                    ui.button(
                        unsafe { stdstr::from_utf8_unchecked(icons::RENAME) },
                    );
                    if ui.is_item_clicked() {
                        i.push_prompt(InputPrompt::Text {
                            label: "Guideline name:".to_string(),
                            default: v.with_glyph(|glyph| {
                                glyph.guidelines[guideline]
                                    .name
                                    .clone()
                                    .unwrap_or("".to_string())
                            }),
                            func: Rc::new(move |editor, string| {
                                editor.with_glyph_mut(|glyph| {
                                    glyph.guidelines[guideline].name = Some(string.clone())
                                });
                            }),
                        });
                    }

                    font_token.pop();
                    custom_button_color.pop();

                    ui.same_line();
                    let mut pop_me = None;
                    if self.selected_idx != Some(guideline) {
                        pop_me =
                            Some(ui.push_style_color(imgui::StyleColor::Button, [0., 0., 0., 0.2]));
                    }
                    ui.button(&im_str);
                    if ui.is_item_clicked() {
                        self.selected_idx = Some(guideline);
                    }
                    if let Some(p) = pop_me {
                        p.pop();
                    }
                }
            });

        if let Some(selected) = self.selected_idx {
            imgui::Window::new(&ui, &imgui::ImString::new("Guideline Inspector"))
                .bg_alpha(1.) 
                .flags(
                    imgui::WindowFlags::NO_RESIZE
                        | imgui::WindowFlags::NO_MOVE
                        | imgui::WindowFlags::NO_COLLAPSE,
                )
                .position([tx, ty - th - 12.], imgui::Condition::Always)
                .size([tw, th], imgui::Condition::Always)
                .build(|| {
                    let (mut at, mut angle) = v.with_glyph(|glyph| {
                        let at = glyph.guidelines[selected].at.clone();
                        let angle = match glyph.guidelines[selected].angle {
                            glifparser::IntegerOrFloat::Integer(n) => n as f32,
                            glifparser::IntegerOrFloat::Float(n) => n,
                        };

                        (at, angle)
                    });

                    let (old_at, old_angle) = (at.clone(), angle.clone());
                    imgui_decimal_text_field("X", ui, &mut at.x);
                    imgui_decimal_text_field("Y", ui, &mut at.y);
                    imgui_decimal_text_field("Angle", ui, &mut angle);

                    if at != old_at || angle != old_angle {
                        v.begin_modification("Modify guideline.");
                        v.with_glyph_mut(|glyph| {
                            glyph.guidelines[selected].at = at;
                            glyph.guidelines[selected].angle = IntegerOrFloat::Float(angle);
                        });
                        v.end_modification();
                    }
                });
        }
    }
}
