// Argument parser
use git_version::git_version;

use clap; //argparse lib

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HeadlessMode {
    None,
    Save,
    Export,
    RunScript, // unused until scripting support added
}

#[derive(Clone, PartialEq, Debug)]
pub struct Args {
    pub filename: Option<String>,
    pub headless_mode: HeadlessMode,
}

pub fn parse_args() -> Args {
    let matches = clap::App::new("MFEKglif")
        .setting(clap::AppSettings::DeriveDisplayOrder)
        .version(git_version!(fallback=env!("CARGO_PKG_VERSION")))
        .author("Fredrick R. Brennan, Matthew Blanchard, MFEK Authors")
        .about("Glyph editor, Modular Font Editor K Project")
        .arg(
            clap::Arg::with_name("GLIF")
                .help("Input UFO format .glif file")
                .index(1),
        )
        .arg(
            clap::Arg::with_name("export")
                .long("export")
                .short("e")
                .takes_value(false)
                .conflicts_with("save")
                .help(r#"Export glyph to current name (you should make a copy if not glifjson format)"#)
        )
        .arg(
            clap::Arg::with_name("save")
                .long("save")
                .short("s")
                .takes_value(false)
                .help(r#"Save glyph to .glifjson"#)
        )
        .get_matches();

    eprint!("This is MFEKglif {} (“{}”), ", env!("CARGO_PKG_VERSION"), env!("MFEK_REL_CODENAME"));
    if atty::is(atty::Stream::Stderr) {
        eprintln!("compiled @ {}", env!("COMPILED_AT"));
    } else {
        eprintln!("compiled @ {}", env!("COMPILED_AT_PLAIN"));
    }

    let mode = match (matches.is_present("export"), matches.is_present("save")) {
        (true, false) => HeadlessMode::Export,
        (false, true) => HeadlessMode::Save,
        _ => HeadlessMode::None,
    };

    Args {
        filename: matches.value_of("GLIF").map(|s| s.to_string()),
        headless_mode: mode,
    }
}
