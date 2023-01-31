use egui::Context;

use crate::{filedialog, editor::Editor, user_interface::Interface};

use super::window::{GlifWindow, WindowManager};

pub fn menu_bar(ctx: &Context, v: &mut Editor, i: &mut Interface, wm: &mut WindowManager) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            //
            // File
            //
            ui.menu_button("File", |ui|{
                if ui.button("Open").clicked() {
                    match filedialog::open_filename(Some("glif,glifjson"), None) {
                        Some(f) => v.load_glif(i, &f),
                        None => {},
                    };
                }
                if ui.button("Save").clicked() {
                    v.save_glif(false);
                }
                if ui.button("Export").clicked() {
                    v.export_glif(Some(i));
                }
                if ui.button("Exit").clicked() {
                    v.quit(i);
                }
            });

            //
            // Edit
            //
            ui.menu_button("Edit", |ui| {
                if ui.button("Undo").clicked() {
                    v.undo();
                }
                if ui.button("Redo").clicked() {
                    v.redo();
                }
            });

            //
            // Windows
            //
            ui.menu_button("Windows", |ui| {
                let mut inspector_open = wm.inspector.open();
                ui.checkbox(&mut inspector_open, "Inspector");
                wm.inspector.set_open(inspector_open);
            })
        })
    }); 
}