use lazy_static::lazy_static;

use crate::constants::CONSOLE_FONTS;

use font_kit::{
    family_name::FamilyName as FKFamilyName, handle::Handle as FKHandle, properties::Properties,
    source::SystemSource,
};

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

pub struct Font {
    pub data: Vec<u8>,
    pub path: Option<PathBuf>,
}

fn load_font(family: &[FKFamilyName]) -> Font {
    log::debug!("Looking for a UI font to satisfy request for {:?}", family);
    let font = match SystemSource::new().select_best_match(family, &Properties::new()) {
        Ok(FKHandle::Path { path, .. }) => Font {
            path: Some(path.clone()),
            data: fs::read(path).expect("Failed to open font path system specified"),
        },
        Ok(FKHandle::Memory { bytes, .. }) => Font {
            path: None,
            data: Arc::try_unwrap(bytes).expect("Failed to load in-memory font"),
        },
        Err(e) => panic!(
            "Failed to select font for {:?} ! Error from fontkit {:?}",
            family, e
        ),
    };
    log::debug!("OK: Found {:?} (len {})", &font.path, font.data.len());
    font
}

lazy_static! {
    pub static ref SYSTEMSANS: Font = load_font(&[
        FKFamilyName::Title("Segoe UI".to_string()),
        FKFamilyName::SansSerif
    ]);
    pub static ref SYSTEMMONO: Font = load_font(CONSOLE_FONTS.as_slice());
}
