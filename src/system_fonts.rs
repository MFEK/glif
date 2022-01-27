use lazy_static::lazy_static;

//use crate::constants::CONSOLE_FONTS;

use font_kit::{
    error::SelectionError::NotFound as FKNotFoundError, family_name::FamilyName as FKFamilyName,
    handle::Handle as FKHandle, properties::Properties, source::SystemSource,
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
    let mut font = None;
    let source = SystemSource::new();
    let props = Properties::new();
    for fkfamname in family {
        let best_match = source.select_best_match(&[fkfamname.clone()], &props);
        if let Err(FKNotFoundError) = best_match {
            log::debug!("Skipped {:?}", fkfamname);
        }
        font = match best_match {
            Ok(FKHandle::Path { path, .. }) => Some(Font {
                path: Some(path.clone()),
                data: fs::read(path).expect("Failed to open font path system specified"),
            }),
            Ok(FKHandle::Memory { bytes, .. }) => Some(Font {
                path: None,
                data: Arc::try_unwrap(bytes).expect("Failed to load in-memory font"),
            }),
            // try next font…
            Err(FKNotFoundError) => continue,
            Err(e) => panic!(
                "Failed to select font for {:?} ! Error from fontkit {:?}",
                family, e
            ),
        };
        if let Some(ref font) = font {
            log::debug!(
                "OK: Found {:?} (matched @ {:?}, len {})",
                &font.path,
                fkfamname,
                font.data.len()
            );
            break;
        }
    }
    match font {
        Some(font) => font,
        None => {
            panic!(
                "In request for {:?}, no matches were made; cannot render UI!",
                family
            );
        }
    }
}

lazy_static! {
    /// Windows 10 comes first because if we allow Windows to match on `sans-serif`, it will give
    /// us Verdana, which looks incongruent on modern Windows OS. So, we specifically ask for Segoe
    /// UI first. Meanwhile, on macOS…the situation is very confusing and complex due to Apple's
    /// difficulty in deciding on a default font. I believe it works to try "Helvetica" and then
    /// ".SFUI-Text" (San Francisco UI Text); Apple does _NOT_ resolve `sans-serif` to anything,
    /// resulting in crash which became issue №220.
    pub static ref SYSTEMSANS: Font = load_font(&[
        // Windows 10
        FKFamilyName::Title("Segoe UI".to_string()),
        // Linux (fontconfig)
        FKFamilyName::SansSerif,
        // old macOS
        FKFamilyName::Title(".SFUIText".to_string()),
        // new macOS (≈2016+)
        FKFamilyName::Title("Helvetica".to_string()),
    ]);
    //TODO: Replace console.
    //pub static ref SYSTEMMONO: Font = load_font(CONSOLE_FONTS.as_slice());
}
