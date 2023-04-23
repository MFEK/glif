use super::super::prelude::*;
use super::{Guidelines, SplitGuidelines};
use crate::tool_behaviors::add_guideline::AddGuideline;
use crate::user_interface::{self, gui::{build_and_add_icon_button, icons}, InputPrompt};
use glifparser::IntegerOrFloat;
use std::rc::Rc;

lazy_static! {
    static ref PLUS_GLOBE: String = format!("{}{}", icons::_PLUS, icons::_GLOBE);
}

impl Guidelines {
    pub fn tool_dialog(&mut self, v: &mut Editor, i: &mut Interface, ui: &mut egui::Ui) {
        let (mut guidelines, _, local_guidelines_len, _) = SplitGuidelines::new(v).as_tuple();

        for (gidx, (guideline, is_global)) in guidelines.iter_mut().enumerate() {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    let (
                        mut guidelines,
                        _guidelines_len,
                        local_guidelines_len,
                        _global_guidelines_len,
                    ) = SplitGuidelines::new(v).as_tuple();

                    if build_and_add_icon_button::<"icons_small">(v, ui, icons::_PLUS, "Add guideline").clicked() {
                        v.push_behavior(Box::new(AddGuideline::new(0., false)));
                    }

                    if build_and_add_icon_button::<"icons_small">(v, ui, &*PLUS_GLOBE, "Add global guideline").clicked() {
                        v.push_behavior(Box::new(AddGuideline::new(0., true)));
                    }

                    if build_and_add_icon_button::<"icons_small">(v, ui, icons::_GLOBE, "Make global").clicked() {
                        v.write_metrics(i);
                    }

                    if let Some(selected) = self.selected_idx {
                        //log::trace!("Selected {}; global len {}", selected, global_guidelines_len);
                        let selected_i = if guidelines[selected].1 {
                            selected - local_guidelines_len
                        } else {
                            selected
                        };
                        //log::trace!("Selected index {}", selected_i);

                        if guidelines[selected].1
                            && guidelines[selected].0.data.as_guideline().format
                        {
                            ui.label("Format defined.");
                        } else {
                            let mb = build_and_add_icon_button::<"icons_small">(v, ui, icons::_MINUS, "Remove guideline");
                            if mb.clicked() {
                                v.begin_modification(
                                    &format!(
                                        "Remove {} guideline.",
                                        if guidelines[selected].1 {
                                            "global"
                                        } else {
                                            "local"
                                        }
                                    ),
                                    false,
                                );
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

                    let is_global = *is_global;
                    let guideline_name = guideline.name.clone().unwrap_or_else(|| {
                        if gidx == 0 {
                            String::from("Unnamed")
                        } else {
                            format!("Unnamed {}", gidx + 1)
                        }
                    });

                    let guideline_display = guideline_name.clone();
                    let mut im_str = guideline_display;

                    if !(is_global && guideline.data.as_guideline().format) {
                        let rb = build_and_add_icon_button::<"icons_small">(v, ui, icons::_RENAME, "Rename guideline");
                        if rb.clicked() {
                            i.push_prompt(InputPrompt::Text {
                                label: "Guideline name:".to_string(),
                                default: guideline_name,
                                func: Rc::new(move |editor, string| {
                                    let gidx = if is_global {
                                        gidx - local_guidelines_len
                                    } else {
                                        gidx
                                    };
                                    editor.begin_modification("Change guideline name.", false);
                                    if is_global {
                                        editor.guidelines[gidx].name = Some(string.clone());
                                    } else {
                                        editor.with_glyph_mut(move |glyph| {
                                            glyph.guidelines[gidx].name = Some(string.clone())
                                        });
                                    }
                                    editor.end_modification();
                                }),
                            });
                        }
                    }

                    if is_global {
                        let (tooltip, label) = if guideline.data.as_guideline().format {
                            ("In UFO",
                            icons::_UFO)
                        } else {
                            ("In fontinfo.plist",
                            icons::_GLOBE)
                        };
                        build_and_add_icon_button::<"icons_small">(v, ui, label, tooltip);
                    }

                    let name_b = ui.button(&im_str);
                    if name_b.clicked() {
                        self.selected_idx = Some(gidx);
                    }
                });
            });
        }

        if let Some(selected) = self.selected_idx {
            ui.horizontal(|ui| {
                let (guidelines, _guidelines_len, local_guidelines_len, _global_guidelines_len) =
                    SplitGuidelines::new(v).as_tuple();
                if guidelines.is_empty() {
                    return;
                }

                let (mut at, mut angle) = {
                    let at = guidelines[selected].0.at;
                    let angle = guidelines[selected].0.angle.into();

                    (at, angle)
                };

                let (old_at, old_angle) = (at, angle);
                if !guidelines[selected].0.data.as_guideline().fixed {
                    user_interface::gui::windows::egui_parsed_textfield(
                        ui,
                        "X",
                        at.x,
                        &mut self.edit_buf,
                    );
                    user_interface::gui::windows::egui_parsed_textfield(
                        ui,
                        "Y",
                        at.y,
                        &mut self.edit_buf,
                    );
                } else {
                    ui.label(format!(
                        "Position of {} is fixed.",
                        guidelines[selected]
                            .0
                            .name
                            .as_ref()
                            .unwrap_or(&format!("Unnamed {}", selected + 1))
                    ));
                }
                if !(guidelines[selected].1 && guidelines[selected].0.data.as_guideline().format) {
                    user_interface::gui::windows::egui_parsed_textfield(
                        ui,
                        "Angle",
                        angle,
                        &mut self.edit_buf,
                    );
                } else {
                    ui.label(format!(
                        "Angle of {} is fixed.",
                        guidelines[selected]
                            .0
                            .name
                            .as_ref()
                            .unwrap_or(&format!("Unnamed {}", selected + 1))
                    ));
                }

                if at != old_at
                    || (!guidelines[selected].0.data.as_guideline().format && angle != old_angle)
                {
                    let selected_i = if guidelines[selected].1 {
                        selected - local_guidelines_len
                    } else {
                        selected
                    };

                    v.begin_modification(
                        &format!(
                            "Modify {} guideline.",
                            if guidelines[selected].1 {
                                "global"
                            } else {
                                "local"
                            }
                        ),
                        false,
                    );
                    if guidelines[selected].1 {
                        v.guidelines[selected_i].at = at;
                        v.guidelines[selected_i].angle = IntegerOrFloat::Float(angle);
                    } else {
                        v.with_glyph_mut(|glyph| {
                            glyph.guidelines[selected_i].at = at;
                            glyph.guidelines[selected_i].angle = IntegerOrFloat::Float(angle);
                        });
                    }
                    if guidelines[selected].0.data.as_guideline().right {
                        v.with_glyph_mut(|glyph| {
                            glyph.width = Some(at.x as u64);
                        });
                    }
                    v.end_modification();
                }
            });
        }
    }
}
