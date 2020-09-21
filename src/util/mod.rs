// Utilities
pub mod argparser;
#[macro_use]
pub mod macros;
pub mod math;

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
macro_rules! debug_event {
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

        if let Some(info) = msg {
            eprintln!("\n{}\n", info.bright_red());
        }

        if let Some(args) = info.message() {
            eprintln!("\n{}\n", args.to_string().bright_red());
        }

        if env::var("RUST_BACKTRACE").is_ok() {
            let mut bt = Backtrace::new();
            bt.resolve();
            eprintln!("Requested backtrace:\n{:?}", bt);
        }
    }));
}

// This prevents debug!() etc from producing mojibake. Yes, really, this is the best solution. :-|
#[cfg(target_family = "windows")]
pub fn set_codepage_utf8() {
    extern crate winapi;
    unsafe {
        debug_assert!(winapi::um::wincon::SetConsoleOutputCP(winapi::um::winnls::CP_UTF8) == 1);
    }
}
