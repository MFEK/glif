use lazy_static::lazy_static;

use crate::constants::CONSOLE_FONTS;

use font_kit::{
    error::SelectionError::NotFound as FKNotFoundError, family_name::FamilyName as FKFamilyName,
    handle::Handle as FKHandle, properties::Properties, source::SystemSource,
};

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct SystemFont {
    pub data: Vec<u8>,
    pub path: Option<PathBuf>,
}

fn load_font(family: &[FKFamilyName]) -> SystemFont {
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
            Ok(FKHandle::Path { path, .. }) => {
                let font_is_scalable = ["otf", "ttf"]
                    .into_iter()
                    .any(|e: &str| path.extension().map(|ee| ee == e).unwrap_or(false));
                // This is possible on GNU/Linux which has BDF fonts.
                if !font_is_scalable {
                    continue;
                }
                Some(SystemFont {
                    path: Some(path.clone()),
                    data: fs::read(path).expect("Failed to open font path system specified"),
                })
            }
            Ok(FKHandle::Memory { bytes, .. }) => Some(SystemFont {
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
    pub static ref SYSTEMSANS: SystemFont = load_font(&[
        // Windows 10
        FKFamilyName::Title("Segoe UI".to_string()),
        // Linux (fontconfig)
        FKFamilyName::SansSerif,
        // old macOS
        FKFamilyName::Title(".SFUIText".to_string()),
        // new macOS (≈2016+)
        FKFamilyName::Title("Helvetica".to_string()),
        // Linux (fallback)
        FKFamilyName::Title("Noto Sans".to_string()),
        FKFamilyName::Title("Roboto".to_string()),
    ]);
    pub static ref SYSTEMSERIF: SystemFont = load_font(&[
        // Windows 10
        FKFamilyName::Title("Times New Roman".to_string()),
        // Linux (fontconfig)
        FKFamilyName::Serif,
        // macOS
        FKFamilyName::Title("Times".to_string()),
        FKFamilyName::Title("TimesNewRomanPSMT".to_string()),
        // Linux (fallback)
        FKFamilyName::Title("FreeSerif".to_string()),
        FKFamilyName::Title("Noto Serif".to_string()),
        FKFamilyName::Title("Roboto Serif".to_string()),
    ]);
    pub static ref SYSTEMMONO: SystemFont = load_font(CONSOLE_FONTS.as_slice());
}
