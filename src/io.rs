
use crate::ipc;
use crate::editor::Editor;

use glifparser::{MFEKGlif, glif::MFEKPointData};
use log::debug;

use std::env;
use std::path::Path;

pub fn load_glif<F: AsRef<Path> + Clone>(v: &mut Editor, filename: F) {
    // TODO: Actually handle errors now that we have them.
    let glif: MFEKGlif<MFEKPointData> = glifparser::read_from_filename(&filename).expect("Invalid glif!").into();

    if env::var("DEBUG_DUMP_GLYPH").is_ok() {
        debug!("{:#?}", &glif.clone());
    }

    v.set_glyph(glif);

    ipc::fetch_metrics(v);

    if let Some(ref mut window) = v.sdl_window {
        window.set_title(&format!("MFEKglif — {}", filename.as_ref().to_str().unwrap() )).expect("Failed to set SDL2 window title");
    }
}
