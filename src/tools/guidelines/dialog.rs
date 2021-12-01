use super::super::prelude::*;
use super::{Guidelines, SplitGuidelines};
use crate::tool_behaviors::add_guideline::AddGuideline;
use crate::user_interface::gui::FONT_IDS;
use crate::user_interface::{self, icons, InputPrompt};
use glifparser::IntegerOrFloat;
use imgui::StyleColor;
use std::rc::Rc;

lazy_static::lazy_static! {
    static ref PLUS_GLOBE: Vec<u8> = icons::chain(&[icons::PLUS, icons::GLOBE]);
}

impl Guidelines {
    pub fn tool_dialog(&mut self, v: &mut Editor, i: &mut Interface, ui: &imgui::Ui) {
        let (tx, ty, tw, th) = i.get_tools_dialog_rect();

        imgui::Window::new(&imgui::ImString::new("Guidelines"))
            .bg_alpha(1.) // See comment on fn redraw_skia
            .flags(
                imgui::WindowFlags::NO_RESIZE
                    | imgui::WindowFlags::NO_MOVE
                    | imgui::WindowFlags::NO_COLLAPSE,
            )
            .position([tx, ty], imgui::Condition::Always)
            .size([tw, th], imgui::Condition::Always)
            .build(ui, || {
                let (mut guidelines, _guidelines_len, local_guidelines_len, global_guidelines_len) = SplitGuidelines::new(v).as_tuple();

                let pop_me = ui.push_style_color(imgui::StyleColor::Button, [0., 0., 0., 0.2]);

                ui.button(
                    unsafe { imgui::ImStr::from_utf8_with_nul_unchecked(icons::PLUS) },
                    [0., 0.],
                );

                if ui.is_item_clicked(imgui::MouseButton::Left) {
                    v.push_behavior(Box::new(AddGuideline::new(0., false)));
                }

                ui.same_line(0.);
                ui.button(
                    unsafe { imgui::ImStr::from_utf8_with_nul_unchecked(&PLUS_GLOBE) },
                    [0., 0.],
                );

                if ui.is_item_clicked(imgui::MouseButton::Left) {
                    v.push_behavior(Box::new(AddGuideline::new(0., true)));
                }

                if let Some(selected) = self.selected_idx {
                    //log::trace!("Selected {}; global len {}", selected, global_guidelines_len);
                    let selected_i = if guidelines[selected].1 {selected - local_guidelines_len} else {selected};
                    //log::trace!("Selected index {}", selected_i);

                    ui.same_line(0.);
                    if guidelines[selected].1 && guidelines[selected].0.data.as_guideline().format {
                        ui.text(imgui::ImString::new("Format defined."));
                    } else {
                        ui.button(
                            unsafe { imgui::ImStr::from_utf8_with_nul_unchecked(icons::MINUS) },
                            [0., 0.],
                        );
                        if ui.is_item_clicked(imgui::MouseButton::Left) {
                            v.begin_modification(&format!("Remove {} guideline.", if guidelines[selected].1 { "global" } else { "local" }));
                            self.selected_idx = None;
                            if guidelines[selected].1 {
                                v.guidelines.remove(selected_i);
                            } else {
                                v.with_glyph_mut(|glyph| {
                                    glyph.guidelines.remove(selected_i);
                                });
                            }
                            v.end_modification();
                        }
                    }
                }

                pop_me.pop(ui);

                ui.separator();

                for (gidx, (guideline, is_global)) in guidelines.iter_mut().enumerate() {
                    let is_global = *is_global;
                    let guideline_name = guideline.name.clone()
                            .unwrap_or_else(|| if gidx == 0 { String::from("Unnamed") } else { format!("Unnamed {}", gidx+1) });

                    let guideline_display = imgui::ImString::new(guideline_name.clone());
                    let im_str = guideline_display;

                    let font_token = ui.push_font(FONT_IDS.with(|ids| ids.borrow()[1]));
                    let custom_button_color = ui.push_style_color(
                        imgui::StyleColor::Button,
                        ui.style_color(StyleColor::WindowBg),
                    );

                    if !(is_global && guideline.data.as_guideline().format) {
                        ui.button(
                            unsafe { imgui::ImStr::from_utf8_with_nul_unchecked(icons::RENAME) },
                            [0., 0.],
                        );
                        if ui.is_item_clicked(imgui::MouseButton::Left) {
                            i.push_prompt(InputPrompt::Text {
                                label: "Guideline name:".to_string(),
                                default: guideline_name,
                                func: Rc::new(
                                    move |editor, string| {
                                        let gidx = if is_global {gidx - local_guidelines_len} else {gidx};
                                        editor.begin_modification("Change guideline name.");
                                        if is_global {
                                            editor.guidelines[gidx].name = Some(string.clone());
                                        } else {
                                            editor.with_glyph_mut(move |glyph|
                                                glyph.guidelines[gidx].name = Some(string.clone())
                                            );
                                        }
                                        editor.end_modification();
                                    }
                                ),
                            });
                        }
                        ui.same_line(0.);
                    }

                    if is_global {
                        ui.text(unsafe { imgui::ImStr::from_utf8_with_nul_unchecked(if guideline.data.as_guideline().format { icons::UFO } else { icons::GLOBE }) });
                    }

                    font_token.pop(ui);
                    custom_button_color.pop(ui);

                    ui.same_line(0.);
                    let mut pop_me = None;
                    if self.selected_idx != Some(gidx) {
                        pop_me =
                            Some(ui.push_style_color(imgui::StyleColor::Button, [0., 0., 0., 0.2]));
                    }
                    ui.button(&im_str, [-1., 0.]);
                    if ui.is_item_clicked(imgui::MouseButton::Left) {
                        self.selected_idx = Some(gidx);
                    }
                    if let Some(p) = pop_me {
                        p.pop(ui);
                    }
                }
            });

        if let Some(selected) = self.selected_idx {
            let (tx, ty, tw, th) = i.get_inspector_dialog_rect();
            imgui::Window::new(&imgui::ImString::new("Guideline Inspector"))
                .bg_alpha(1.)
                .flags(
                    imgui::WindowFlags::NO_RESIZE
                        | imgui::WindowFlags::NO_MOVE
                        | imgui::WindowFlags::NO_COLLAPSE,
                )
                .position([tx, ty], imgui::Condition::Always)
                .size([tw, th], imgui::Condition::Always)
                .build(ui, || {
                    let (guidelines, _guidelines_len, local_guidelines_len, _global_guidelines_len) = SplitGuidelines::new(v).as_tuple();
                    if guidelines.is_empty() { return }

                    let (mut at, mut angle) = {
                        let at = guidelines[selected].0.at;
                        let angle = guidelines[selected].0.angle.into();

                        (at, angle)
                    };

                    let (old_at, old_angle) = (at, angle);
                    if !guidelines[selected].0.data.as_guideline().fixed {
                        user_interface::util::imgui_decimal_text_field("X", ui, &mut at.x, None);
                        user_interface::util::imgui_decimal_text_field("Y", ui, &mut at.y, None);
                    } else {
                        ui.text(imgui::im_str!("Position of {} is fixed.", guidelines[selected].0.name.as_ref().unwrap_or(&format!("Unnamed {}", selected+1))));
                    }
                    if !(guidelines[selected].1 && guidelines[selected].0.data.as_guideline().format) {
                        user_interface::util::imgui_decimal_text_field("Angle", ui, &mut angle, None);
                    } else {
                        ui.text(imgui::im_str!("Angle of {} is fixed.", guidelines[selected].0.name.as_ref().unwrap_or(&format!("Unnamed {}", selected+1))));
                    }

                    if at != old_at || (!guidelines[selected].0.data.as_guideline().format && angle != old_angle) {
                        let selected_i = if guidelines[selected].1 {selected - local_guidelines_len} else {selected};

                        v.begin_modification(&format!("Modify {} guideline.", if guidelines[selected].1 { "global" } else { "local" }));
                        if guidelines[selected].1 {
                            v.guidelines[selected_i].at = at;
                            v.guidelines[selected_i].angle = IntegerOrFloat::Float(angle);
                        } else {
                            v.with_glyph_mut(|glyph| {
                                glyph.guidelines[selected_i].at = at;
                                glyph.guidelines[selected_i].angle = IntegerOrFloat::Float(angle);
                            });
                        }
                        v.end_modification();
                    }
                });
        }
    }
}
