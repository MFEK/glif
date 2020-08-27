// Argument parser

use clap; //argparse lib

pub struct Args {
    pub filename: String
}

pub fn parse_args() -> Args {
    let matches = clap::App::new("Qglif")
        .version(&*format!("{}-alpha", git_version!()))
        .about("Glyph editor, Modular Font Editor Q Project")
        .arg(clap::Arg::with_name("GLIF")
             .help("Input UFO format .glif file")
             .required(true)
             .index(1))
        .get_matches();
    Args { filename: matches.value_of("GLIF").unwrap().to_string() }
}
