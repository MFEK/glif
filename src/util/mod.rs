// Utilities
pub mod argparser;
pub mod math;

use std::env;

use std::panic::set_hook;

use backtrace::Backtrace;
use colored::Colorize;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref DEBUG: bool = option_env!("DEBUG").is_some();
    pub static ref DEBUG_EVENTS: bool = option_env!("DEBUG_EVENTS").is_some();
}


#[macro_export]
///! Given a field on the State struct, and an enumerator that implements IntoEnumIterator, cycle
///! through its variants and update state. An optional condition is provided. $state is expected to
///! be an inner thread::LocalKey<State>.
macro_rules! trigger_toggle_on {
    ($state:ident, $state_var:ident, $enum:ident, $cond:expr) => {
        let $state_var = $state.viewport.$state_var;
        if $cond {
            let mut e = $enum::into_enum_iter()
                .cycle()
                .skip(1 + $state_var as usize);
            let n = e.next().unwrap();
            $state.viewport.$state_var = n;
        }
    };
    ($state:ident, $state_var:ident, $enum:ident) => {
        trigger_toggle_on!($state, $state_var, $enum, true);
    };
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
    unsafe {
        debug_assert!(winapi::um::wincon::SetConsoleOutputCP(winapi::um::winnls::CP_UTF8) == 1);
    }
}
