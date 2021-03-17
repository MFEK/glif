use lazy_static::lazy_static;

use crate::renderer::constants::CONSOLE_FONTS;

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
    let source = SystemSource::new();
    let font = source
        .select_best_match(family, &Properties::new())
        .unwrap();
    match font {
        FKHandle::Path { path, .. } => Font {
            path: Some(path.clone()),
            data: fs::read(path).expect("Failed to open font path system specified"),
        },
        FKHandle::Memory { bytes, .. } => Font {
            path: None,
            data: Arc::try_unwrap(bytes).expect("Failed to load in-memory font"),
        },
    }
}

lazy_static! {
    pub static ref SYSTEMSANS: Font = load_font(&[
        FKFamilyName::Title("Segoe UI".to_string()),
        FKFamilyName::SansSerif
    ]);
    pub static ref SYSTEMMONO: Font = load_font(CONSOLE_FONTS.as_slice());
}
