use super::EguiManager;

use egui::{FontData, FontTweak};

use crate::constants;
use crate::system_fonts::*;

fn tweak(font: FontData) -> FontData {
    font.tweak(FontTweak {
        scale: constants::FONT_SCALE_FACTOR,
        ..Default::default()
    })
}

fn tweak_enlarge(font: FontData) -> FontData {
    font.tweak(FontTweak {
        scale: 1.4 * constants::FONT_SCALE_FACTOR,
        ..Default::default()
    })
}

impl EguiManager {
    #[rustfmt::skip]
    pub fn set_system_fonts(&mut self) {
        let ctx = &self.egui.egui_ctx;
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.iter_mut().map(move |s|{let new = tweak((s.1).clone()); *s.1 = new;}).for_each(drop);
        fonts.font_data.insert("sans".to_owned(), tweak(egui::FontData::from_static(&SYSTEMSANS.data)));
        fonts.font_data.insert("serif".to_owned(), tweak(egui::FontData::from_static(&SYSTEMSERIF.data)));
        fonts.font_data.insert("mono".to_owned(), tweak(egui::FontData::from_static(&SYSTEMMONO.data)));
        fonts.font_data.insert("icons".to_owned(), tweak_enlarge(egui::FontData::from_static(&ICONSFONT.data)));
        fonts.font_data.insert("icons_small".to_owned(), tweak(egui::FontData::from_static(&ICONSFONT.data)));
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "sans".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().insert(0, "mono".to_owned());
		fonts.families.insert(egui::FontFamily::Name("icons".into()), vec!["icons".to_owned(), "mono".to_owned()]);
		fonts.families.insert(egui::FontFamily::Name("icons_small".into()), vec!["icons_small".to_owned(), "mono".to_owned()]);
        ctx.set_fonts(fonts);
    }
}
