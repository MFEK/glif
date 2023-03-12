#![allow(unused)]

#[macro_use]
extern crate cfg_if;
#[macro_use]
extern crate cargo_emit;
#[macro_use]
extern crate git_version;

use which::which;

use std::process::Command;

fn main() {
    cfg_if! {
        if #[cfg(all(feature = "sdl2-static", feature = "sdl2-dynamic"))] {
            compile_error!("Features `sdl2-static` and `sdl2-dynamic` are mutually exclusive!");
        }
    }
    let version = git_version!(
        args = ["--tags", "--always", "--dirty=-desync"],
        fallback = "v1"
    );
    rustc_env!("MFEK_VERSION", "{}", version);
    rustc_env!("MFEK_COMPILED_AT", "{}", chrono::Local::now().timestamp());
    #[cfg(all(target_os = "macos", feature = "sdl2-dynamic"))]
    linker_macos_sdl2();
    cfg_if! {
        if #[cfg(feature = "reproducible-build")] {
            rustc_cfg!("reproducible");
        } else if #[cfg(feature = "sdl2-static")] {
            cfg_if! {
                if #[cfg(target_os = "linux")] {
                    warning!("Trying to statically link SDL2 is known to be buggy on Linux. Proceed with caution.");
                }
            }
        } else if #[cfg(all(not(feature = "sdl2-static"), any(target_family = "windows", target_family = "macos"), not(feature = "sdl2-dynamic")))] {
            println!("cargo:rustc-cfg=feature=\"sdl2-static\"");
        } else if #[cfg(all(target_family = "windows", feature = "sdl2-dynamic"))] {
            warning!("It is neither recommended nor supported to compile MFEKglif on Windows w/dynamic SDL2.");
        }
    }
}

#[cfg(all(target_os = "macos", feature = "sdl2-dynamic"))]
fn linker_macos_sdl2() {
    // sdl2-config --libs
    let sdl2config_bin = which("sdl2-config").expect("`sdl2-config` not in PATH");
    let c = Command::new(sdl2config_bin)
        .arg("--libs")
        .output()
        .expect("Broken SDL2 installation");
    let o = std::str::from_utf8(&c.stdout).expect("Broken SDL2 installation");
    for a in o.split(" ") {
        log::debug!("Setting linker arg {}", a);
        cargo_emit::rustc_link_arg!(a);
    }
}
