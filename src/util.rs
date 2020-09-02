// Utilities
pub mod argparser;

use crate::glifparser::Codepoint;
use std::fmt;
use std::option::Option;
use std::panic::set_hook;

lazy_static! {
    pub static ref DEBUG: bool = { option_env!("DEBUG").is_some() };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ({
        use crate::util::DEBUG;
        if *DEBUG {
            eprint!("Debug: ");
            eprintln!($($arg)*);
        }
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
            Self::Hex(c) => {
                let cc = *c as u32;
                fmt::LowerHex::fmt(&cc, f)
            }
            Self::Undefined => fmt::LowerHex::fmt(&-1, f),
        }
    }
}
