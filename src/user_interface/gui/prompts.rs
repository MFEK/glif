use std::borrow::BorrowMut;

use egui::Align2;

//use super::{IMGUI_RESERVE, PROMPT_CLR, PROMPT_STR, TOOLBOX_HEIGHT, TOOLBOX_WIDTH};
use crate::editor::Editor;
use crate::user_interface::{InputPrompt, Interface};

use super::PROMPT_STR;

pub fn build_and_check_prompts(v: &mut Editor, i: &mut Interface, ctx: &egui::Context) {
    match i.peek_prompt().clone() {
        InputPrompt::YesNo { question, afterword, func } => {
            egui::Window::new("MFEKglif")
                .resizable(false)
                .collapsible(false)
                .anchor(Align2::CENTER_CENTER, [0., 0.])
                .fixed_pos( egui::Pos2::new(
                    (i.viewport.winsize.0 / 2.) * i.os_dpi(),
                    (i.viewport.winsize.1 / 2.) * i.os_dpi()
                ))
                .show(ctx, |ui| {
                    ui.label(format!("{}", question));

                    ui.vertical_centered(|ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Yes").clicked() {
                                func(v, i, true);
                                i.pop_prompt();
                            }
    
                            if ui.button("No").clicked() {
                                func(v, i, false);
                                i.pop_prompt();
                            }
                        });
                    });
                });
            
        },
        InputPrompt::Text { label, default, func } => {
            egui::Window::new(label)
                .resizable(false)
                .collapsible(false)
                .anchor(Align2::CENTER_CENTER, [0., 0.])
                .fixed_pos( egui::Pos2::new(
                    (i.viewport.winsize.0 / 2.) * i.os_dpi(),
                    (i.viewport.winsize.1 / 2.) * i.os_dpi()
                ))
                .show(ctx, |ui| {
                    PROMPT_STR.with(|prompt_str| {
                        let buffer = prompt_str.borrow_mut().to_string();

                        if ui.text_edit_singleline(&mut buffer).lost_focus() {
                            func(v, buffer);
                            i.pop_prompt();
                        }

                        prompt_str.replace(buffer);
                    })
                });
        },
    }
}
