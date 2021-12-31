use super::Editor;

use std::process;

use crate::util::argparser::{Args, HeadlessMode};

impl Editor {
    pub fn headless(&mut self, args: &Args) -> ! {
        match args.filename {
            Some(ref filename) => self.load_glif_headless(&filename),
            None => panic!("Cannot go headless without a .glif file to work on"),
        }

        if args.no_contour_ops {
            let glyph = self.glyph.as_mut().unwrap();
            for layer in glyph.layers.iter_mut() {
                for contour in layer.outline.iter_mut() {
                    contour.operation = None;
                }
            }
        }

        match args.headless_mode {
            HeadlessMode::None => panic!("Headless called on non-headless editor!"),
            HeadlessMode::Save => {
                self.save_glif(false).unwrap();
            }
            HeadlessMode::Export => {
                self.export_glif(None).unwrap();
            }
            HeadlessMode::Flatten => {
                self.flatten_glif(None, false).unwrap();
            }
            HeadlessMode::RunScript => unimplemented!(),
        }
        process::exit(0)
    }
}
