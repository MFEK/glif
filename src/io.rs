use crate::state::Glyph;

use crate::events;
use crate::state;
use crate::STATE;
use glifparser::Glif;
use log::debug;
use std::cell::RefCell;
use std::env;
use std::fs;
use std::path::Path;

pub fn load_glif<F: AsRef<Path> + Clone>(filename: F) {
    let glif =
        glifparser::read_ufo_glif(&fs::read_to_string(&filename).expect("Failed to read file"));

    if env::var("DEBUG_DUMP_GLYPH").is_ok() {
        debug!("{:#?}", glif);
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

pub fn save_glif(v: &RefCell<crate::state::State<Option<crate::state::PointData>>>) {
    let mut _v = v.borrow_mut();
    let _glyph = _v.glyph.as_ref().unwrap();
    let filename: std::path::PathBuf = _glyph.filename.clone();

    let mut lib = _glyph
        .glif
        .lib
        .as_ref()
        .unwrap_or(&xmltree::Element::new("lib"))
        .clone();
    let vws_lib = &super::events::vws::generate_lib(_v.vws_contours.clone());

    if let Some(vws) = vws_lib {
        for child in &vws.children {
            lib.children.push(child.clone());
        }
    }

    let glif_string = {
        let glif = &_glyph.glif;

        let new_glif = Glif {
            anchors: glif.anchors.clone(),
            format: glif.format,
            lib: Some(lib),
            name: glif.name.clone(),
            order: glif.order,
            outline: glif.outline.clone(),
            unicode: glif.unicode,
            width: glif.width,
        };

        glifparser::write_ufo_glif(&new_glif)
    };

    fs::write(filename, glif_string).expect("Unable to write file");
}

use crate::filedialog;

pub fn export_glif(v: &RefCell<crate::state::State<Option<crate::state::PointData>>>) {
    let cur_file = v.borrow().glyph.as_ref().unwrap().filename.clone();
    let filename = filedialog::save_filename(Some("glif"), None);
    v.borrow_mut().mode = state::Mode::Select;
    events::vws::export_vws(filename.unwrap_or(cur_file));
}
