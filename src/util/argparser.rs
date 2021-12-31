// Argument parser
use git_version::git_version;

use clap; //argparse lib

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HeadlessMode {
    None,
    Flatten,
    Export,
    Save,
    RunScript, // unused until scripting support added
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Args {
    pub filename: Option<String>,
    pub headless_mode: HeadlessMode,
    pub no_contour_ops: bool,
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
            clap::Arg::with_name("flatten")
                .long("flatten")
                .short("f")
                .takes_value(false)
                .conflicts_with_all(&["save", "export"])
                .help(r#"Flatten glyph to current name (you should make a copy if not glifjson format)"#)
        )
        .arg(
            clap::Arg::with_name("export")
                .long("export")
                .short("e")
                .takes_value(false)
                .help(r#"Export glyph to current name (you should make a copy if not glifjson format)"#)
        )
        .arg(
            clap::Arg::with_name("save")
                .long("save")
                .short("s")
                .takes_value(false)
                .help(r#"Save glyph to .glifjson"#)
        )
        .arg(
            clap::Arg::with_name("no-contour-ops")
                .long("no-contour-ops")
                .short("N")
                .takes_value(false)
                .help(r#"For either the save or the export operation, remove all contour operations, don't apply them."#)
        )
        .get_matches();

    eprint!(
        "This is MFEKglif v{} (“{}”)",
        env!("CARGO_PKG_VERSION"),
        env!("MFEK_REL_CODENAME")
    );
    if let Some(time) = option_env!("COMPILED_AT_PLAIN") {
        eprintln!(", compiled @ {}", time);
    } else {
        eprintln!(".");
    }

    let headless_mode = if matches.is_present("export") {
        HeadlessMode::Export
    } else if matches.is_present("flatten") {
        HeadlessMode::Flatten
    } else if matches.is_present("save") {
        HeadlessMode::Save
    } else {
        HeadlessMode::None
    };

    let no_contour_ops = matches.is_present("no-contour-ops");

    Args {
        filename: matches.value_of("GLIF").map(|s| s.to_string()),
        headless_mode,
        no_contour_ops,
    }
}
