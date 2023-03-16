use egui::{Align2, Color32, Context, Stroke, Ui};

use crate::{constants::*, editor::Editor, tools::ToolEnum, user_interface::Interface};

pub fn build_button(v: &mut Editor, ui: &mut Ui, text: &str, te: ToolEnum) {
    let stroke = if v.get_tool() == te {
        Stroke {
            width: 2.0,
            color: Color32::from_rgb(255, 190, 0),
        }
    } else {
        Stroke::NONE
    };

    let button = egui::Button::new(text)
        .stroke(stroke)
        .min_size(egui::vec2(32., 32.));

    let respone = ui.add(button);
    if respone.clicked() {
        v.set_tool(te);
    }

    respone.on_hover_text(format!("{:?}", te));
}
pub fn tool_bar(ctx: &Context, v: &mut Editor, _i: &mut Interface) {
    egui::Window::new("Tools")
        .anchor(Align2::LEFT_TOP, [16., 31. * FONT_SCALE_FACTOR])
        .title_bar(false)
        .default_width(10. * FONT_SCALE_FACTOR)
        .min_width(15. * FONT_SCALE_FACTOR)
        .resizable(false)
        .enabled(!v.is_modifying())
        // .shadow(None) Help!: How can we remove the shadow? - elih
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
