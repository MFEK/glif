use crate::filedialog;
use crate::ipc;
use crate::editor::Editor;

use mfek_ipc;
use glifparser::{MFEKGlif, glif::MFEKPointData};
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

    if mfek_ipc::module_available("MFEKmetadata".into()) == mfek_ipc::Available::Yes {
        ipc::fetch_metrics(v);
    }

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
