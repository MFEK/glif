use egui::Context;
use glifrenderer::toggles::PreviewMode;

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
            // View
            //
            ui.menu_button("View", |ui| {
                ui.menu_button("Mode", |ui| {
                    let mut edit = i.viewport.preview_mode == PreviewMode::None;
                    ui.checkbox(&mut edit, "None");

                    if edit {
                        i.viewport.preview_mode = PreviewMode::None;
                    }

                    let mut selected = i.viewport.preview_mode == PreviewMode::NoUnselectedPoints;
                    ui.checkbox(&mut selected, "Fill");

                    if selected {
                        i.viewport.preview_mode = PreviewMode::NoUnselectedPoints;
                    }

                    let mut paper = i.viewport.preview_mode == PreviewMode::Paper;
                    ui.checkbox(&mut paper, "Paper");

                    if paper {
                        i.viewport.preview_mode = PreviewMode::Paper;
                    }
                });
                
                ui.checkbox(&mut i.grid.show, "Grid");
            });

            //
            // Windows
            //
            ui.menu_button("Windows", |ui| {
                let mut inspector_open = wm.inspector.open();
                ui.checkbox(&mut inspector_open, "Inspector");
                wm.inspector.set_open(inspector_open);

                let mut grid_open = wm.grid.open();
                ui.checkbox(&mut grid_open, "Grid");
                wm.grid.set_open(grid_open);
            })
        })
    }); 
}