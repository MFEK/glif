use crate::{
    constants::*,
    editor::Editor,
    tools::ToolEnum,
    user_interface::{
        gui::{build_icon_button, IntoButtonResponse as _},
        Interface,
    },
};
use egui::{Align2, Color32, Context, Stroke, Ui};

use super::icons;

fn build_button<'a>(v: &mut Editor, ui: &mut Ui, text: &str, te: ToolEnum) {
    // Must build button outside bind_response so that v can move into the closure.
    let mut b = build_icon_button::<"icons">(v, ui, text, te.to_string().as_str());

    let stroke = if v.get_tool().to_string() == te.to_string() {
        Stroke {
            width: 2.0,
            color: Color32::from_rgb(255, 190, 0),
        }
    } else {
        Stroke::NONE
    };

    let mut button = b.button.unwrap().stroke(stroke);
    b.button = Some(button);

    let response = b.egui_response(ui);

    let mut bind_response = move |response: egui::Response| {
        if response.clicked() {
            v.set_tool(te);
        }
    };
    bind_response(response)
}

pub fn tool_bar(ctx: &Context, v: &mut Editor, _i: &mut Interface) {
    egui::Window::new("Tools")
        .anchor(Align2::LEFT_TOP, [16., 31. * FONT_SCALE_FACTOR])
        .title_bar(false)
        .default_width(10. * FONT_SCALE_FACTOR)
        .min_width(15. * FONT_SCALE_FACTOR)
        .resizable(false)
        .enabled(!v.is_modifying())
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                build_button(v, ui, icons::PAN, ToolEnum::Pan);
                build_button(v, ui, icons::SELECT, ToolEnum::Select);
                build_button(v, ui, icons::PEN, ToolEnum::Pen);
                ui.separator();
                build_button(v, ui, icons::ZOOM, ToolEnum::Zoom);
                build_button(v, ui, icons::MEASURE, ToolEnum::Measure);
                ui.separator();
                build_button(v, ui, icons::VWS, ToolEnum::VWS);
                build_button(v, ui, icons::PAP, ToolEnum::PAP);
                build_button(v, ui, icons::DASH, ToolEnum::Dash);
                ui.separator();
                build_button(v, ui, icons::ANCHOR, ToolEnum::Anchors);
                build_button(v, ui, icons::SHAPES, ToolEnum::Shapes);
                build_button(v, ui, icons::IMAGES, ToolEnum::Image);
                build_button(v, ui, icons::GUIDELINES, ToolEnum::Guidelines);
            })
        });
}
