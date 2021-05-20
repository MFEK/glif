use super::Editor;

use glifparser::Glif;
use glifparser::glif::{self, mfek::{MFEKGlif, MFEKPointData, Layer}};
use log;
use plist::{self, Value as PlistValue};

use std::{fs, io, path, process};

use crate::filedialog;
use crate::io as glif_io;

impl Editor {
    pub fn flatten_glif(&mut self) {
        self.mark_preview_dirty();
        self.rebuild();
        let export = self.prepare_export();
        let layer = &export.layers[0];
        if export.layers.len() > 1 {
            log::warn!("In a flatten operation, layers not in the topmost group will be discarded and not in your chosen file. You may want to export (Ctrl+E) and not flatten.");
        }
        
        let glif_struct = self.glyph.as_ref().unwrap().to_exported(&layer);
        let target = filedialog::save_filename(Some("glif"), None).map(|f| {
            glif::write_to_filename(&glif_struct, &f).map(|()|log::info!("Requested flatten to {:?}", &f)).unwrap_or_else(|e|panic!("Failed to write glif: {:?}", e));
        }).unwrap_or_else(||log::warn!("Requested flatten cancelled due to failed dialog"));
    }

    pub fn export_glif(&mut self) {
        self.mark_preview_dirty();
        self.rebuild();
        let glif_fn = self.with_glyph(|g|g.filename.as_ref().unwrap().file_name().unwrap().to_owned());
        let glif_name = self.with_glyph(|g|g.name.clone());
        let ipc_info = self.ipc_info.clone().expect("Cannot export w/o IPC data");

        // `self.preview` contains flattened versions of all the layers, which are always cubic Bézier
        // splines. We know it's Some(_) because we rebuilt above.
        //
        // In the first phase, we iterate flattened layer groups ("previews") and write the glyph
        // data.
        let export = self.prepare_export();

        let font_pb = if let Some(ref font) = ipc_info.font {
            Some(font.clone())
        } else if export.layers.len() == 1 {
            None
        } else {
            panic!("Glyph has {} layers; font must have a parent UFO!", self.get_layer_count())
        };

        for (i, layer) in export.layers.iter().enumerate() {
            if !layer.visible { continue }

            let target_dir = layer.to_glyphs_dir(i);

            let mut target = self.glyph.as_ref().unwrap().filename.as_ref().unwrap().clone();

            match font_pb {
                Some(ref pb) => {
                    target = pb.clone();
                    target.push(&target_dir);
                    match fs::create_dir(&target) {
                        Err(e) => {
                            if e.kind() != io::ErrorKind::AlreadyExists {
                                panic!("{:?}", e);
                            }
                        },
                        Ok(()) => ()
                    }
                    target.push(&glif_fn);
                    log::info!("Targeting {:?} to write {}", &target, &layer.name);
                },
                None => ()
            }

            let glif_struct = self.glyph.as_ref().unwrap().to_exported(&layer);
            glif::write_to_filename(&glif_struct, &target).unwrap_or_else(|e|panic!("Failed to write glif: {:?}", e));

            if font_pb.is_none() {
                log::warn!("Exported .glif without a parent UFO font. Cannot create layer(info|contents).plist.");
                if layer.color.is_some() {
                    log::error!(".glif's layer 0 calls for a color, but it has no parent UFO. Cannot create layercontents.plist, color will be lost!")
                }
                return
            }

            // In the second phase, we write the plist files layerinfo.plist and
            // layercontents.plist. We have to make sure that they either (a) don't exist or (b)
            // exist and are compatible. If incompatible, we emit a warning.
            use glifparser::glif::mfek::layer::ToLayerInfoPlist;
            // layerinfo.plist
            let needs_layerinfo = layer.color.is_some();
            if font_pb.is_none() { panic!("Cannot write layerinfo for glyph; not in a font!") }
            if needs_layerinfo {
                let mut layerinfo = target.parent().expect("Cannot write layerinfo for glyph; at root of filesystem???").to_owned();
                layerinfo.push("layerinfo.plist");
                log::debug!("We are going to try to write a layerinfo.plist to {:?}", &layerinfo);
                let mut layerinfo_p = None;
                let mut current_layerinfo_p = None;
                if path::Path::exists(&layerinfo) {
                    log::info!("Layer already has layerinfo, checking compatibility");
                    current_layerinfo_p = Some(PlistValue::from_file(&layerinfo).expect(&format!("Failed to deserialize layerinfo.plist in {:?}", &layerinfo)));
                }

                let layerinfo_plist = layer.to_layerinfo_plist();
                layerinfo_plist.map(|layerinfo_p_| {
                    current_layerinfo_p.map(|cmp| {
                        if layerinfo_p_ != cmp {
                            log::warn!("I am replacing an existing layerinfo.plist with an incompatible layerinfo.plist. Other glyphs in this font may appear in different colors! This is not a bug in MFEKglif, you must use unique names for layers that will be differently colored *across* your font.");
                        }
                    });
                    layerinfo_p = Some(layerinfo_p_);
                });

                if let Some(li) = layerinfo_p {
                    li.to_file_xml(layerinfo).expect(&format!("Failed to write layerinfo.plist for layer {} in glyph {}", i, &glif_name));
                    log::info!("Wrote layer {} of glyph {}'s layerinfo.plist. Color was {}", i, &glif_name, layer.color.unwrap().to_string());
                }
            }
        }

        // layercontents.plist
        use glifparser::glif::mfek::layer::ToLayerContentsPlist;
        let mut our_layercontents = (&export.layers).as_slice().to_layercontents_plist();
        let layercontents = font_pb.clone();
        if let Some(mut layercontents_f) = layercontents {
            layercontents_f.push("layercontents.plist");
            if path::Path::exists(&layercontents_f) {
                let current_layercontents_p = Some(PlistValue::from_file(&layercontents_f).expect(&format!("Failed to deserialize layercontents.plist in {:?}", &layercontents_f)));
                our_layercontents = (&export.layers).as_slice().merge_layercontents_plists(current_layercontents_p.unwrap());
            }
            our_layercontents.to_file_xml(&layercontents_f).expect(&format!("Failed to write layercontents.plist for glyph {}", &glif_name));
            log::info!("Wrote glyph {}'s layercontents.plist.", &glif_name);
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
        ret.width = self.width;
        ret.name = self.name.clone();
        ret.unicode = self.unicode.clone();
        ret.filename = self.filename.clone();
        ret
    }
}
