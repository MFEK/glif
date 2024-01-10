mod error;
pub use error::SystemFontError;

use lazy_static::lazy_static;
use log::warn;

use font_kit::{
    error::SelectionError::{
        self as FKSelectionError, CannotAccessSource as FKSourceError, NotFound as FKNotFoundError,
    },
    handle::Handle as FKHandle,
    source::SystemSource,
};

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct SystemFont {
    pub data: Vec<u8>,
    pub path: Option<PathBuf>,
}

impl TryInto<SystemFont> for FKHandle {
    type Error = SystemFontError;
    fn try_into(self: FKHandle) -> Result<SystemFont, SystemFontError> {
        match self {
            FKHandle::Path { path, .. } => {
                let font_is_scalable = ["otf", "ttf"]
                    .into_iter()
                    .any(|e: &str| path.extension().map(|ee| ee == e).unwrap_or(false));
                // This is possible on GNU/Linux which has BDF fonts.
                if !font_is_scalable {
                    return Err(SystemFontError::NotScalable);
                }
                Ok(SystemFont {
                    path: Some(path.clone()),
                    data: fs::read(path)?,
                })
            }
            FKHandle::Memory { bytes, .. } => Ok(SystemFont {
                path: None,
                data: Arc::try_unwrap(bytes)?,
            }),
        }
    }
}

fn load_font(family: &[&str]) -> Result<SystemFont, FKSelectionError> {
    log::debug!("Looking for a UI font to satisfy request for {:?}", family);
    let source = SystemSource::new();
    let mut font: Option<SystemFont> = None;
    let mut last_err = None;
    let mut best_match;
    for fkfamname in family {
        best_match = source.select_by_postscript_name(&fkfamname);
        if let Err(FKNotFoundError) = best_match {
            log::debug!("Skipped {:?}", fkfamname);
            last_err = Some(best_match.clone().unwrap_err());
        }
        font = match best_match {
            Ok(f) => f.try_into().ok(),
            // try next font…
            Err(FKNotFoundError) => continue,
            Err(FKSourceError) => {
                warn!("I/O error when trying to access font {}!", fkfamname);
                continue;
            }
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
        Some(font) => Ok(font),
        None => Err(last_err.unwrap()),
    }
}

lazy_static! {
    pub static ref DEFAULTSERIF: SystemFont = (FKHandle::Memory { bytes: Arc::from(include_bytes!("../../resources/fonts/Besley-Regular.ttf").to_vec()), font_index: 0 }).try_into().unwrap();
    pub static ref DEFAULTSANS: SystemFont = (FKHandle::Memory { bytes: Arc::from(include_bytes!("../../resources/fonts/MFEKSans-Regular.ttf").to_vec()), font_index: 0 }).try_into().unwrap();
    pub static ref DEFAULTMONO: SystemFont = (FKHandle::Memory { bytes: Arc::from(include_bytes!("../../resources/fonts/TT2020Base-Regular.ttf").to_vec()), font_index: 0 }).try_into().unwrap();

    /// Windows 10 comes first because if we allow Windows to match on `sans-serif`, it will give
    /// us Verdana, which looks incongruent on modern Windows OS. So, we specifically ask for Segoe
    /// UI first. Meanwhile, on macOS…the situation is very confusing and complex due to Apple's
    /// difficulty in deciding on a default font. I believe it works to try "Helvetica" and then
    /// ".SFUI-Text" (San Francisco UI Text); Apple does _NOT_ resolve `sans-serif` to anything,
    /// resulting in crash which became issue №220.
    pub static ref SYSTEMSANS: SystemFont = load_font(&[
        // Windows 10 & 11
        "SegoeUI",
        // Windows XP
        "Verdana",
        // Linux (fontconfig)
        "sans-serif",
        // old macOS
        ".SFUIText",
        // new macOS (≈2016+)
        "helvetica",
        // Linux (fallback)
        "NotoSans-Regular",
        "Roboto-Regular"
    ]).unwrap_or(DEFAULTSANS.clone());
    pub static ref SYSTEMSERIF: SystemFont = load_font(&[
        // Windows 10
        "TimesNewRomanPSMT",
        // Linux (fontconfig)
        "serif",
        // macOS
        "Times",
        "Adobe-Times",
        // Linux (fallback)
        "FreeSerif",
        "NotoSerif-Regular",
        "RobotoSerifNormalRoman_500wght",
    ]).unwrap_or(DEFAULTSERIF.clone());
    pub static ref SYSTEMMONO: SystemFont = load_font(&[
        "Inconsolata-Regular",
        "Consolas",
        "CourierNewPSMT",
        "Adobe-Courier",
        "Courier10PitchBT-Roman"
    ]).unwrap_or(DEFAULTMONO.clone());
    pub static ref ICONSFONT: SystemFont = (FKHandle::Memory { bytes: Arc::from(include_bytes!("../../resources/fonts/icons.otf").to_vec()), font_index: 0 }).try_into().unwrap();
}
