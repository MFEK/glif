use crate::state::{state, Glyph};
use std::fs;
use std::path::Path;

pub fn load_glif<F: AsRef<Path> + Clone>(filename: F) {
    let glif =
        glifparser::read_ufo_glif(&fs::read_to_string(&filename).expect("Failed to read file"));
    state.with(|v| {
        v.borrow_mut().glyph = Some(Glyph {
            glif,
            filename: filename.as_ref().to_path_buf(),
        })
    });
    state.with(|v| {
        v.borrow().glyph.as_ref().map(|glyph| {
            let glif = &glyph.glif;
            debug!(
                "Loaded {:?} (U+{:04x}) from {}",
                glif.name,
                glif.unicode,
                state
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
