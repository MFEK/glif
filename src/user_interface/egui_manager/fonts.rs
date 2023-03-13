use super::EguiManager;

use egui::{FontData, FontDefinitions, FontTweak};

use crate::constants;
use crate::system_fonts::*;

fn tweak(font: FontData) -> FontData {
    font.tweak(FontTweak {scale: constants::FONT_SCALE_FACTOR, .. Default::default()})
}

impl EguiManager {
    pub fn set_system_fonts(&mut self) {
        let ctx = &self.egui.egui_ctx;
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.iter_mut().map(move |s|{let new = tweak((s.1).clone()); *s.1 = new;}).for_each(drop);
        fonts.font_data.insert("sans".to_owned(), tweak(egui::FontData::from_static(&SYSTEMSANS.data)));
        fonts.font_data.insert("serif".to_owned(), tweak(egui::FontData::from_static(&SYSTEMSERIF.data)));
        fonts.font_data.insert("mono".to_owned(), tweak(egui::FontData::from_static(&SYSTEMMONO.data)));
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "sans".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().insert(0, "mono".to_owned());
        ctx.set_fonts(fonts);
    }
}
