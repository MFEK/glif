// Utilities
pub mod math;

use crate::editor::events::EditorEvent;
use crate::editor::headless::IS_HEADLESS;

use std::fs;
use std::panic::set_hook;
use std::{env, process};

use colored::Colorize;
use glifparser::PointData;
use lazy_static::lazy_static;
use log;
use msgbox::IconType;

lazy_static! {
    pub static ref DEBUG_DUMP_GLYPH: bool = env::var("MFEK_DEBUG_DUMP_GLYPH").is_ok();
    pub static ref DEBUG_EVENTS: bool = env::var("MFEK_DEBUG_EVENTS").is_ok();
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

pub fn log_sdl_event(event: &sdl2::event::Event) {
    if *DEBUG_EVENTS {
        log::debug!("Got SDL event: {:?}", &event);
    }
}

pub fn log_editor_event(event: &EditorEvent) {
    if *DEBUG_EVENTS {
        log::debug!("Got editor event: {:?}", &event);
    }
}

pub fn hard_error(msg: &str) -> ! {
    eprintln!("{}", msg.bright_red());
    process::exit(1)
}

fn now_epoch() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn quit_next_frame() {
    log::warn!("Quitting immediately due to Ctrl-C received at console (or SIGTERM signal)…");
    std::process::exit(1);
}

pub fn set_panic_hook() {
    set_hook(Box::new(|info| {
        let headless = IS_HEADLESS.with(|h| *h.borrow());

        let msg = info
            .payload()
            .downcast_ref::<String>()
            .map(|s| s.clone())
            .unwrap_or_else(|| info.to_string());

        if env::var("MFEK_QUIET_CRASH").is_err() {
            eprintln!("\n{}\n", msg.bright_red());
        }

        let quiet_msgbox =
            env::var("MFEK_QUIET_CRASH").is_err() && env::var("MFEK_QUIET_CRASH_MSGBOX").is_err();
        if !headless && !quiet_msgbox {
            let err = msgbox::create(
                "Uh oh! \u{2014} MFEKglif crashed",
                info.to_string().as_str(),
                IconType::Error,
            );

            match err {
                Ok(_) => log::trace!("Opened crash msgbox successfully"),
                Err(e) => log::error!("Failed to create error box! {:?}", e),
            }
        }

        let mut bt = backtrace::Backtrace::new();
        bt.resolve();

        if env::var("RUST_BACKTRACE").is_ok() && env::var("MFEK_QUIET_CRASH").is_err() {
            eprintln!("Requested backtrace:\n{:?}", bt);
        }

        if env::var("MFEK_BACKTRACE_NO_WRITE").is_err() && env::var("MFEK_QUIET_CRASH").is_err() {
            let mut pb = env::temp_dir();
            pb.push(format!("error_log_{}", now_epoch()));
            pb.set_extension("txt");

            let err = fs::write(pb.clone(), format!("{:?}", bt));

            match err {
                Ok(_) => log::info!("Wrote backtrace to {}", pb.display()),
                Err(_) => log::error!("Failed to write backtrace to file! {}", pb.display()),
            }
        }
    }));
}

pub fn init_env_logger() {
    if env::var("RUST_LOG").is_err() {
        if *DEBUG_DUMP_GLYPH || *DEBUG_EVENTS {
            env::set_var(
                "RUST_LOG",
                "INFO,MFEKglif=trace,rafx_framework=off,rafx_api=off,skulpin=off",
            )
        } else {
            env::set_var(
                "RUST_LOG",
                "INFO,rafx_framework=off,rafx_api=off,skulpin=off",
            )
        }
    }
    env_logger::init();
}

// This prevents debug!() etc from producing mojibake. Yes, really, this is the best solution. :-|
#[cfg(target_family = "windows")]
pub fn set_codepage_utf8() {
    unsafe {
        debug_assert!(winapi::um::wincon::SetConsoleOutputCP(winapi::um::winnls::CP_UTF8) == 1);
    }
}

#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct MFEKGlifGuidelineInfo {
    pub fixed: bool,
    pub format: bool,
    pub right: bool,
}
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum MFEKGlifPointData {
    Guideline(MFEKGlifGuidelineInfo),
}
impl Default for MFEKGlifPointData {
    fn default() -> Self {
        MFEKGlifPointData::Guideline(MFEKGlifGuidelineInfo::default())
    }
}
impl MFEKGlifPointData {
    pub fn new_guideline_data(fixed: bool, format: bool, right: bool) -> Self {
        Self::Guideline(MFEKGlifGuidelineInfo {
            fixed,
            format,
            right,
        })
    }
    pub fn as_guideline(&self) -> MFEKGlifGuidelineInfo {
        #[allow(irrefutable_let_patterns)]
        if let MFEKGlifPointData::Guideline(guide) = self {
            *guide
        } else {
            panic!("Tried to unwrap non-guideline as guideline")
        }
    }
}
impl PointData for MFEKGlifPointData {}
