// Utilities
pub mod argparser;

use std::panic::set_hook;
use std::option::Option;
use std::fmt;
use crate::glifparser::Codepoint;

lazy_static!(pub static ref DEBUG: bool = { option_env!("DEBUG").is_some() }; );

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ({
        use crate::util::DEBUG;
        eprint!("Debug: ");
        if *DEBUG { eprintln!($($arg)*); }
    })
}

pub fn set_panic_hook() {
    set_hook(Box::new(|info| {
        println!("{:?}", info.message().unwrap());
    }));
}

// Trait necessary as f32 is primitive
pub trait RoundFloat {
    fn fround(self, digits: u8) -> Self;
}

impl RoundFloat for f32 {
    fn fround(self, digits: u8) -> Self {
        (self * 100.).round() / 100.
    }
}

impl fmt::LowerHex for Codepoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> { 
        match self {
            Self::Hex(c) => { let cc = *c as u32; fmt::LowerHex::fmt(&cc, f)},
            Self::Undefined => {fmt::LowerHex::fmt(&-1, f)}
        }
    }
}
