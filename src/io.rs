use crate::state::Glyph;
use crate::util::DEBUG;
use crate::STATE;
use std::env;
use std::fs;
use std::path::Path;

pub fn load_glif<F: AsRef<Path> + Clone>(filename: F) {
    let glif =
        glifparser::read_ufo_glif(&fs::read_to_string(&filename).expect("Failed to read file"));

    if env::var("DEBUG_DUMP_GLYPH").is_ok() {
        debug!("{:?}", glif);
    }

    STATE.with(|v| {
        v.borrow_mut().glyph = Some(Glyph {
            glif,
            filename: filename.as_ref().to_path_buf(),
            guidelines: Vec::new(),
        })
    });

    STATE.with(|v| {
        v.borrow().glyph.as_ref().map(|glyph| {
            let glif = &glyph.glif;
            debug!(
                "Loaded {:?} (U+{:04x}) from {}",
                glif.name,
                glif.unicode,
                STATE
                    .with(|v| v
                        .borrow()
                        .glyph
                        .as_ref()
                        .expect("Glyph NULL!?")
                        .filename
                        .clone())
                    .display()
            );
        });
    });
}
