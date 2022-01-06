use super::Editor;
use crate::args::HeadlessMode;

use std::cell::RefCell;
use std::process;

thread_local!(pub static IS_HEADLESS: RefCell<bool> = RefCell::new(false));

impl Editor {
    pub fn headless(&mut self) {
        if self.args.headless_mode == HeadlessMode::None {
            return;
        } else {
            IS_HEADLESS.with(|h| *h.borrow_mut() = true);
        }

        let filename = match &self.args.filename {
            Some(filename) => filename.clone(),
            None => panic!("Cannot go headless without a .glif file to work on"),
        };
        self.load_glif_impl(filename);

        if self.args.no_contour_ops {
            let glyph = self.glyph.as_mut().unwrap();
            for layer in glyph.layers.iter_mut() {
                for contour in layer.outline.iter_mut() {
                    contour.operation = None;
                }
            }
        }

        match self.args.headless_mode {
            HeadlessMode::None => unreachable!(),
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
