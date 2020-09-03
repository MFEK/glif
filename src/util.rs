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

