use crate::editor::Editor;
use crate::ipc;
use crate::user_interface::Interface;
use crate::util::DEBUG_DUMP_GLYPH;

use glifparser::{glif::MFEKPointData, MFEKGlif};
use log::debug;

use std::path::Path;

pub fn load_glif<F: AsRef<Path> + Clone>(v: &mut Editor, i: &mut Interface, filename: F) {
    i.set_window_title(&format!(
        "MFEKglif — {}",
        filename.as_ref().to_str().unwrap()
    ))
    .expect("Failed to set SDL2 window title");
    load_glif_headless(v, filename);
}

pub fn load_glif_headless<F: AsRef<Path> + Clone>(v: &mut Editor, filename: F) {
    // TODO: Actually handle errors now that we have them.
    let glif: MFEKGlif<MFEKPointData> = glifparser::read_from_filename(&filename)
        .expect("Invalid glif!")
        .into();

    if *DEBUG_DUMP_GLYPH {
        debug!("{:#?}", &glif.clone());
    }

    v.set_glyph(glif);

    ipc::fetch_metrics(v);
}
