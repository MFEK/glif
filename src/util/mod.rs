// Utilities
pub mod argparser;

use crate::glifparser::Codepoint;
use std::env;
use std::fmt;
use std::option::Option;
use std::panic::set_hook;

use backtrace::Backtrace;
use colored::Colorize;

lazy_static! {
    pub static ref DEBUG: bool = { option_env!("DEBUG").is_some() };
    pub static ref DEBUG_EVENTS: bool = { option_env!("DEBUG_EVENTS").is_some() };
}

#[macro_export]
macro_rules! debug_events {
    ($($arg:tt)*) => ({
        use crate::util::DEBUG_EVENTS;
        if *DEBUG_EVENTS {
            debug!($($arg)*);
        }
    })
}

pub fn set_panic_hook() {
    set_hook(Box::new(|info| {
        let msg = info.payload().downcast_ref::<&str>();

        match msg {
            Some(info) => eprintln!("\n{}\n", info.bright_red()),
            _ => {}
        }

        if env::var("RUST_BACKTRACE").is_ok() {
            let mut bt = Backtrace::new();
            bt.resolve();
            eprintln!("Requested backtrace:\n{:?}", bt);
        }
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
