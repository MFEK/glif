use crate::editor::Editor;

use glifparser::{Glif, MFEKGlif, glif::MFEKPointData};
use log::debug;
use std::{env, fs};
use std::path::Path;

pub fn load_glif<F: AsRef<Path> + Clone>(v: &mut Editor, filename: F) {
    // TODO: Actually handle errors now that we have them.
    let glif: MFEKGlif<MFEKPointData> = glifparser::read_from_filename(&filename).expect("Invalid glif!").into();

    if env::var("DEBUG_DUMP_GLYPH").is_ok() {
        debug!("{:#?}", &glif.clone());
    }

    v.set_glyph(glif);

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
        let filename: std::path::PathBuf = glyph.filename.clone().unwrap();

        let glif_string = {
            glifparser::write(&glyph.clone().into())
        };
    
        fs::write(filename, glif_string.unwrap()).expect("Unable to write file");
    });
}

use crate::filedialog;

pub fn export_glif(v: &Editor) {

    let cur_file = v.with_glyph(|glyph| { glyph.filename.clone() });
    let filename = filedialog::save_filename(Some("glif"), None);
}
