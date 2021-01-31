// Argument parser

use clap; //argparse lib

pub struct Args {
    pub filename: Option<String>,
}

pub fn parse_args() -> Args {
    let matches = clap::App::new("MFEKglif")
        .version(&*format!("{}-alpha", git_version!()))
        .about("Glyph editor, Modular Font Editor K Project")
        .arg(
            clap::Arg::with_name("GLIF")
                .help("Input UFO format .glif file")
                .index(1),
        )
        .get_matches();
    Args {
        filename: matches.value_of("GLIF").map(|s| s.to_string()),
    }
}
