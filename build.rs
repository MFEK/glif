#[macro_use]
extern crate cfg_if;
#[macro_use]
extern crate cargo_emit;
#[macro_use]
extern crate git_version;

use chrono;

fn main() {
    cfg_if! {
        if #[cfg(all(feature = "sdl2-static", feature = "sdl2-dynamic"))] {
            compile_error!("Features `sdl2-static` and `sdl2-dynamic` are mutually exclusive!");
        }
    }
    let version = git_version!(
        args = ["--tags", "--always", "--dirty=-desync"],
        fallback = concat!("v", env!("CARGO_PKG_VERSION"))
    );
    rustc_env!("MFEK_VERSION", "{}", version);
    rustc_env!("MFEK_COMPILED_AT", "{}", chrono::Local::now().timestamp());
    cfg_if! {
        if #[cfg(feature = "reproducible-build")] {
            rustc_cfg!("reproducible");
        } else if #[cfg(feature = "sdl2-static")] {
            cfg_if! {
                if #[cfg(target_os = "linux")] {
                    warning!("Trying to statically link SDL2 is known to be buggy on Linux. Proceed with caution.");
                }
            }
        } else if #[cfg(all(any(not(feature = "sdl2-static"), any(target_family = "windows", target_family = "macos")), not(feature = "sdl2-dynamic")))] {
            rustc_cfg!("features=\"{}\"", "sdl2-static");
        } else if #[cfg(all(target_family = "windows", feature = "sdl2-dynamic"))] {
            warning!("It is neither recommended nor supported to compile MFEKglif on Windows w/dynamic SDL2.");
        }
    }
}
