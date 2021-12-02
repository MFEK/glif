use super::Editor;

use std::process;

use crate::util::argparser::{Args, HeadlessMode};

impl Editor {
    pub fn headless(&mut self, args: &Args) -> ! {
        match args.filename {
            Some(ref filename) => self.load_glif_headless(&filename),
            None => panic!("Cannot go headless without a .glif file to work on"),
        }
        match args.headless_mode {
            HeadlessMode::None => panic!("Headless called on non-headless editor!"),
            HeadlessMode::Save => {
                self.save_glif(false).unwrap();
            }
            HeadlessMode::Export => self.export_glif(None),
            HeadlessMode::RunScript => unimplemented!(),
        }
        process::exit(0)
    }
}
