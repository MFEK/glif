use crate::editor::{Glyph, Editor};

use crate::editor;
use glifparser::MFEKGlif;
use log::debug;
use editor::PointData;
use std::env;
use std::fs;
use std::path::Path;

pub fn load_glif<F: AsRef<Path> + Clone>(v: &mut Editor, filename: F) {
    let glifxml = fs::read_to_string(&filename).expect("Failed to read file");

    // TODO: Actually handle errors now that we have them.
    let mut glif: MFEKGlif<PointData> = glifparser::read(&glifxml).expect("Invalid glif!").into();

    // This is necessary because the glif format doesn't require that a glif have an outline. But
    // we require a place to store contours if the user draws any.
    for layer in &mut glif.layers {
        if layer.outline.is_none() {
            layer.outline = Some(glifparser::Outline::new());
        }
    }

    if env::var("DEBUG_DUMP_GLYPH").is_ok() {
        debug!("{:#?}", &glif.clone());
    }

    v.set_glyph(Glyph {
        glif,
        filename: filename.as_ref().to_path_buf(),
        guidelines: Vec::new(),
    });

    /* 
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
    */
}

pub fn save_glif(v: &mut Editor) {
    v.with_glyph(|glyph| {
        let filename: std::path::PathBuf = glyph.filename.clone();

        let glif_string = {
            // TODO: glifparser::write(&glyph.glif)
        };
    
        //TODO: fs::write(filename, glif_string).expect("Unable to write file");
    });
}

use crate::filedialog;

pub fn export_glif(v: &Editor) {

    let cur_file = v.with_glyph(|glyph| { glyph.filename.clone() });
    let filename = filedialog::save_filename(Some("glif"), None);
}
