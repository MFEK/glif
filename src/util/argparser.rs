// Argument parser
use git_version::git_version;

use clap; //argparse lib

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HeadlessMode {
    None,
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
        .version(&*format!("{}-alpha", git_version!(fallback=env!("CARGO_PKG_VERSION"))))
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
                .help(r#"Export glyph to current name (you should make a copy so as not to lose MFEK data)"#)
        )
        .get_matches();
    Args {
        filename: matches.value_of("GLIF").map(|s| s.to_string()),
        headless_mode: matches
            .is_present("export")
            .then(|| HeadlessMode::Export)
            .unwrap_or_else(|| HeadlessMode::None),
    }
}
