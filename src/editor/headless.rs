use super::Editor;

use std::process;

use crate::io as glif_io;
use crate::util::argparser::{Args, HeadlessMode};

impl Editor {
    pub fn headless(&mut self, args: &Args) -> ! {
        match args.filename {
            Some(ref filename) => glif_io::load_glif(self, &filename),
            None => panic!("Cannot go headless without a .glif file to work on")
        }
        match args.headless_mode {
            HeadlessMode::None => panic!("Headless called on non-headless editor!"),
            HeadlessMode::Export => self.export_glif(),
            HeadlessMode::RunScript => unimplemented!()
        }
        process::exit(0)
    }
}
