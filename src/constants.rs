#![allow(clippy::unusual_byte_groupings)]
/* Factors */
pub static SCALE_FACTOR: f32 = 0.05;
pub static OFFSET_FACTOR: f32 = 10.;
pub static BIG_OFFSET_FACTOR: f32 = OFFSET_FACTOR * 10.;
pub static TINY_OFFSET_FACTOR: f32 = 1.;

// List of fonts to try in order until we find the console font.
// Warning: This is not (yet?) a fallback list. The first found font will be used for all glyphs
// for now.
use font_kit::family_name::FamilyName as FKFamilyName;
use lazy_static::lazy_static;
lazy_static! {
    pub static ref CONSOLE_FONTS: Vec<FKFamilyName> = vec![
        FKFamilyName::Title("Inconsolata".to_string()),
        FKFamilyName::Title("Consolas".to_string()),
        FKFamilyName::Monospace
    ];
}

// CONSOLE
pub const CONSOLE_TEXT_SIZE: f32 = 14.;
pub const CONSOLE_PADDING_X: f32 = CONSOLE_TEXT_SIZE - (CONSOLE_TEXT_SIZE / 3.);
pub const CONSOLE_PADDING_Y_TOP: f32 = 3.;
pub const CONSOLE_PADDING_Y_BOTTOM: f32 = CONSOLE_PADDING_Y_TOP / 2.;
pub static CONSOLE_FILL: u32 = 0xff_000000;
pub static CONSOLE_TEXT_FILL: u32 = 0xff_ffffff;
pub static _CONSOLE_TEXT_ERROR_FILL: u32 = 0xff_ff0000;
