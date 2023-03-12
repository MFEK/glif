use egui::{Align2, Color32, Context, Stroke, Ui};

use crate::{editor::Editor, tools::ToolEnum, user_interface::Interface};

pub fn build_button(v: &mut Editor, ui: &mut Ui, text: &str, te: ToolEnum) {
    let stroke = if v.get_tool() == te {
        Stroke {
            width: 2.0,
            color: Color32::from_rgb(9, 82, 128),
        }
    } else {
        Stroke::NONE
    };

    let button = egui::Button::new(text)
        .stroke(stroke)
        .min_size(egui::vec2(22., 15.));

    let respone = ui.add(button);
    if respone.clicked() {
        v.set_tool(te);
    }

    respone.on_hover_text(format!("{:?}", te));
}
pub fn tool_bar(ctx: &Context, v: &mut Editor, _i: &mut Interface) {
    egui::Window::new("Tools")
        .anchor(Align2::LEFT_TOP, [0., 25.])
        .title_bar(false)
        .default_width(10.)
        .min_width(15.)
        .resizable(false)
        .enabled(!v.is_modifying())
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                build_button(v, ui, "✋", ToolEnum::Pan);
                build_button(v, ui, "⬈", ToolEnum::Select);
                build_button(v, ui, "✒", ToolEnum::Pen);
                ui.separator();
                build_button(v, ui, "🔎", ToolEnum::Zoom);
                build_button(v, ui, "📏", ToolEnum::Measure);
                ui.separator();
                build_button(v, ui, "⚡", ToolEnum::VWS);
                build_button(v, ui, "〰", ToolEnum::PAP);
                build_button(v, ui, "—", ToolEnum::Dash);
                ui.separator();
                build_button(v, ui, "⚓", ToolEnum::Anchors);
                build_button(v, ui, "⏺", ToolEnum::Shapes);
                build_button(v, ui, "🖻", ToolEnum::Image);
                build_button(v, ui, "|", ToolEnum::Guidelines);
            })
        });
}
