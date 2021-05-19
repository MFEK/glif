use super::Editor;

use glifparser::Glif;
use glifparser::glif::{self, mfek::{MFEKGlif, MFEKPointData, Layer}};
use log;
use std::fs;
use std::process;

use crate::io as glif_io;
use crate::util::argparser::{Args, HeadlessMode};

impl Editor {
    pub fn do_headless(&mut self, args: &Args) -> ! {
        match args.filename {
            Some(ref filename) => glif_io::load_glif(self, &filename),
            None => panic!("Cannot go headless without a .glif file to work on")
        }
        match args.headless_mode {
            HeadlessMode::None => panic!("Headless called on non-headless editor!"),
            HeadlessMode::Export => self.do_headless_export(&args),
            HeadlessMode::RunScript => unimplemented!()
        }
        process::exit(0)
    }

    pub fn do_headless_export(&mut self, args: &Args) {
        self.mark_preview_dirty();
        self.rebuild();
        let glif_fn = self.with_glyph(|g|g.filename.as_ref().unwrap().file_name().unwrap().to_owned());
        let ipc_info = self.ipc_info.clone().expect("Cannot export w/o IPC data");
        let font_pb = if let Some(ref font) = ipc_info.font {
            Some(font.clone())
        } else if self.get_layer_count() == 1 {
            None
        } else {
            panic!("Glyph has {} layers; font must have a parent UFO!", self.get_layer_count())
        };

        // preview contains flattened versions of all the layers, which are always cubic Bézier
        // splines. We know it's Some(_) because we rebuilt above.
        for (i, layer) in self.preview.as_ref().unwrap().layers.iter().enumerate() {
            if !layer.visible { continue }

            let target_dir = layer.to_glyphs_dir(i);

            let mut target = self.glyph.as_ref().unwrap().filename.as_ref().unwrap().clone();

            match font_pb {
                Some(ref pb) => {
                    if i != 0 {
                        target = pb.clone();
                        target.push(&target_dir);
                        fs::create_dir(&target);
                        target.push(&glif_fn);
                    }
                    log::info!("Targetting {:?} to write {}", &target, &layer.name);
                },
                None => ()
            }

            let glif_struct = self.glyph.as_ref().unwrap().to_exported(&layer);
            glif::write_to_filename(&glif_struct, &target);
        }
    }
} 

pub trait ExportLayer {
    fn to_exported(&self, layer: &Layer<MFEKPointData>) -> Glif<MFEKPointData>;
}

/// Warning: You should always use this from MFEKglif with glif.preview.layers. If you use it with
/// the normal MFEKGlif type's layers, then you will need to apply contour operations yourself!
impl ExportLayer for MFEKGlif<MFEKPointData> {
    fn to_exported(&self, layer: &Layer<MFEKPointData>) -> Glif<MFEKPointData> {
        let contours: Vec<_> = layer.outline.iter().map(|c| c.inner.clone()).collect();
        let mut ret = Glif::new();
        ret.outline = Some(contours);
        ret.anchors = self.anchors.clone();
        let images = layer.images.iter().map(|(img, matrix)| { let mut img = img.clone(); img.set_matrix(*matrix); img }).collect();
        ret.images = images;
        ret.components = self.components.clone();
        ret.filename = self.filename.clone();
        ret
    }
}
