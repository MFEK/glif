use egui::{Style, Context, Id, Pos2};

use crate::{editor::Editor, user_interface::Interface};

pub fn canvas_context(ctx: &Context, v: &mut Editor, i: &mut Interface) {
    let pos = i.context.unwrap();
    let location = Some(Pos2::new(pos.0, pos.1));
    egui::popup::show_tooltip_at(ctx, Id::from("CanvasContext"), location, |ui| {
        ui.button("TEST");
        ui.button("TWO");
    });
}