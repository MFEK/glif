use crate::{constants::*, editor::Editor};
use egui::{Button, Response, Ui};

use std::cell::RefCell;

pub struct IconButton<const EGUI_FONT_NAME: &'static str> {
    pub button: Option<Button>,
    mutated_ui: bool,
    tooltip: String
}

thread_local!(
    static EGUI_DEFAULT_BUTTON_FONT_ID: RefCell<egui::FontId> =
        RefCell::new(egui::FontId::default());
);

impl<const W: &'static str> IconButton<W> {
    fn new<'a>(_v: &'a mut Editor, ui: &'a mut Ui, text: &'a str, tooltip: &'a str) -> IconButton<W> {
        let use_button_font = text
            .chars()
            .nth(0)
            .map(u32::from)
            .map(|c| c >= 0xF000 && c <= 0xF037)
            .unwrap_or(false);
        let icons_font_family = egui::FontFamily::Name(W.into());
        let icons_font_id = egui::FontId {
            family: icons_font_family.clone(),
            ..Default::default()
        };
        let size = ui.fonts(|f| f.row_height(&icons_font_id));
        let s = ui.style_mut();
        let d = s.text_styles.get(&egui::TextStyle::Button).unwrap().clone();
        EGUI_DEFAULT_BUTTON_FONT_ID.with_borrow_mut(|f| *f = d);
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
        let button = Some(egui::Button::new(text).min_size(egui::vec2(
            size * FONT_SCALE_FACTOR,
            size * FONT_SCALE_FACTOR,
        )));

        let mutated_ui = use_button_font;

        IconButton { button, mutated_ui, tooltip: tooltip.to_owned() }
    }
}

pub trait IntoResponse {
    fn egui_response(&mut self, ui: &mut Ui) -> Response;
}

impl<const W: &'static str> IntoResponse for IconButton<W> {
    fn egui_response(&mut self, ui: &mut Ui) -> Response {
        let button = self.button.take();
        let response = ui.add(button.unwrap());
        if self.mutated_ui {
            let s = ui.style_mut();
            *(s.text_styles.get_mut(&egui::TextStyle::Button).unwrap()) =
                EGUI_DEFAULT_BUTTON_FONT_ID.with_borrow(|fid| (fid.clone()));
            let s = s.to_owned();
            ui.set_style(s.to_owned());
        }
        let response = response.on_hover_text(format!("{}", self.tooltip.as_str()));
        response
    }
}

pub fn build<'a, const W: &'static str>(v: &mut Editor, ui: &mut Ui, text: &str, tooltip: &str) -> IconButton<W> {
    let button = IconButton::<W>::new(v, ui, text, tooltip);
    button
}

pub fn build_and_add<'a, const W: &'static str>(
    v: &mut Editor,
    ui: &'a mut Ui,
    text: &str,
    tooltip: &str
) -> Response {
    let mut button = IconButton::<W>::new(v, ui, text, tooltip);
    button.egui_response(ui)
}
