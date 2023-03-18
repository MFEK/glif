use egui::{Align2, Color32, Context, Stroke, Ui};
use lazy_static::lazy_static;

use super::super::icons;

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

    let use_button_font = text
        .chars()
        .nth(0)
        .map(u32::from)
        .map(|c| c >= 0xF000 && c <= 0xF037)
        .unwrap_or(false);
    let icons_font_family = egui::FontFamily::Name("icons".into());
    let icons_font_id = egui::FontId {
        family: icons_font_family.clone(),
        ..Default::default()
    };
    let size = ui.fonts(|f| f.row_height(&icons_font_id));
    let EGUI_DEFAULT_BUTTON_FONT_ID: egui::FontId = {
        let s = ui.style_mut();
        s.text_styles.get(&egui::TextStyle::Button).unwrap().clone()
    };
    if use_button_font {
        let s = ui.style_mut();
        s.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId {
                family: icons_font_family,
                size,
            },
        );
    }
    {
        let button = egui::Button::new(text)
            .stroke(stroke)
            .min_size(egui::vec2(22. * FONT_SCALE_FACTOR, 30. * FONT_SCALE_FACTOR));

        let response = ui.add(button);
        if response.clicked() {
            v.set_tool(te);

            response.on_hover_text(format!("{:?}", te));
        }
    }
    if use_button_font {
        let s = ui.style_mut();
        *(s.text_styles.get_mut(&egui::TextStyle::Button).unwrap()) = EGUI_DEFAULT_BUTTON_FONT_ID;
        let s = s.to_owned();
        ui.set_style(s.to_owned());
    }
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
