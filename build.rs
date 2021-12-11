use std::str as stdstr;

use strip_ansi_escapes::strip as strip_ansi;

fn main() {
    let profile = std::env::var("PROFILE").unwrap();
    let kyou = include_str!(".cargo/kyou");
    let kyou_plain = strip_ansi(include_str!(".cargo/kyou"));
    if profile == "dev" {
        println!("cargo:rustc-env=debug");
    }
    println!("cargo:rustc-env=COMPILED_AT={}", &kyou);
    println!(
        "cargo:rustc-env=COMPILED_AT_PLAIN={}",
        stdstr::from_utf8(&kyou_plain.unwrap()).unwrap()
    );
}
