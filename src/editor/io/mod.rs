use super::{events::*, Editor};

use MFEKmath::mfek::ResolveCubic;
use glifparser::glif::contour::MFEKContourCommon;
//use fs2::FileExt as _; # TODO: Add file locking.
use glifparser::glif::mfek::{Layer, MFEKGlif};
use glifparser::{Glif, MFEKPointData};
use log;
use mfek_ipc::IPCInfo;
use plist;

use std::{
    ffi::OsString as Oss,
    fs, io,
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::filedialog;
use crate::user_interface::{InputPrompt, Interface};
use crate::util::DEBUG_DUMP_GLYPH;

impl Editor {
    pub fn just_saved(&self) -> bool {
        self.history
            .undo_stack
            .last()
            .map(|undo| {
                undo.description == "Saved glyph"
                    || undo.description == "Flattened glyph"
                    || undo.description == "Exported glyph"
            })
            .unwrap_or(false)
    }

    pub fn has_unsaved_changes(&self) -> bool {
        !self.just_saved() && self.history.undo_stack.last().is_some()
    }

    pub fn filename_or_panic(&self) -> PathBuf {
        self.with_glyph(|g| g.filename.as_ref().unwrap().clone())
    }

    pub fn load_glif<F: AsRef<Path> + Clone>(&mut self, interface: &mut Interface, filename: F) {
        interface
            .set_window_title(&format!(
                "MFEKglif — {}",
                filename.as_ref().to_str().unwrap()
            ))
            .expect("Failed to set SDL2 window title");
        self.load_glif_impl(filename.clone());
        self.mark_preview_dirty();
        self.rebuild(interface);
        self.dispatch_editor_event(
            interface,
            EditorEvent::IOEvent {
                event_type: IOEventType::FileSwitched,
                path: filename.as_ref().to_path_buf(),
            },
        );
    }

    pub fn load_glif_impl<F: AsRef<Path> + Clone>(&mut self, file: F) {
        // TODO: Actually handle errors now that we have them.
        let glif: MFEKGlif<_> = {
            let ext = file.as_ref().extension().map(|e| e.to_ascii_lowercase());
            let ext_or = ext
                .unwrap_or(Oss::from("glif"))
                .to_string_lossy()
                .into_owned();
            let tempglif: MFEKGlif<_> = match ext_or.as_str() {
                "glifjson" => {
                    serde_json::from_str(&fs::read_to_string(&file).expect("Could not open file"))
                        .expect("Could not deserialize JSON MFEKGlif")
                }
                "glif" => glifparser::read_from_filename(&file)
                    .expect("Invalid glif!")
                    .into(),
                _ => {
                    log::error!(
                        "Refusing to write to a file with extension {}: {:?}",
                        ext_or,
                        file.as_ref()
                    );
                    return;
                }
            };

            tempglif
        };

        if *DEBUG_DUMP_GLYPH {
            log::debug!("{:#?}", &glif);
        }

        self.set_glyph(glif);
        self.initialize();
    }

    pub fn save_glif(&mut self, rename: bool) -> Result<PathBuf, ()> {
        self.begin_modification("Saved glyph");
        let res = self.with_glyph_mut(|glyph| {
            let filename: PathBuf = if rename {
                match filedialog::save_filename(Some("glifjson"), None) {
                    Some(f) => f,
                    None => return Err(()),
                }
            } else {
                let mut temp = glyph.filename.clone().unwrap();
                if temp.extension().unwrap() == "glif" {
                    temp.set_extension("glifjson");
                } else if temp.extension().unwrap() != "glifjson" {
                    panic!("Unhandled file extension");
                }
                temp
            };

            glyph.filename = None;
            log::info!("Requested save to {:?}", &filename);
            fs::write(&filename, serde_json::to_vec_pretty(&glyph).unwrap()).expect("Write failed");
            glyph.filename = Some(filename.clone());
            Ok(filename)
        });
        self.end_modification();
        res
    }

    pub fn flatten_glif(
        &mut self,
        interface: Option<&mut Interface>,
        rename: bool,
    ) -> Result<PathBuf, ()> {
        self.mark_preview_dirty();
        if let Some(i) = interface {
            self.rebuild(i);
        }
        
        let mut export = self.prepare_export();
        if export.layers.len() > 1 {
            log::warn!("In a flatten operation, layers not in the topmost group will be discarded and not in your chosen file. You may want to export (Ctrl+E) and not flatten.");
        }
        let layer = &mut export.layers[0];


        let glif_struct = self.glyph.as_ref().unwrap().to_exported(layer);

        let filename = self.with_glyph(|glyph| {
            let mut filename: PathBuf = if rename {
                match filedialog::save_filename(Some("glif"), None) {
                    Some(f) => f,
                    None => return None,
                }
            } else {
                glyph.filename.clone().unwrap()
            };

            filename.set_extension("glif");

            glifparser::write_to_filename(&glif_struct, &filename)
                .map(|()| log::info!("Requested flatten to {:?}", &filename))
                .unwrap_or_else(|e| panic!("Failed to write glif: {:?}", e));

            Some(filename)
        });
        match filename {
            Some(filename) => {
                self.begin_modification("Flattened glyph");
                self.end_modification();
                Ok(filename)
            }
            None => Err(()),
        }
    }

    pub fn export_glif(&mut self, interface: Option<&mut Interface>) -> Result<(), ()> {
        self.mark_preview_dirty();
        if let Some(i) = interface {
            self.rebuild(i);
        }
        let glif_fn = {
            let mut temp = self.filename_or_panic();
            temp.set_extension("glif");
            temp.file_name().unwrap().to_owned()
        };
        let glif_name = self.with_glyph(|g| g.name.clone());
        let ipc_info = self.ipc_info.clone().unwrap_or(IPCInfo::default());

        // `self.preview` contains flattened versions of all the layers, which are always cubic Bézier
        // splines. We know it's Some(_) because we rebuilt above.
        //
        // In the first phase, we iterate flattened layer groups ("previews") and write the glyph
        // data.
        let mut export = self.prepare_export();

        let font_pb = if let Some(ref font) = ipc_info.font {
            Some(font.clone())
        } else if export.layers.len() == 1 {
            None
        } else {
            log::error!(
                "Glyph has {} layers; font must have a parent UFO!",
                self.get_layer_count()
            );
            return Err(());
        };

        for (i, layer) in export.layers.iter_mut().enumerate() {
            if !layer.visible {
                continue;
            }

            let target_dir = layer.to_glyphs_dir(i);

            let mut target = self
                .glyph
                .as_ref()
                .unwrap()
                .filename
                .as_ref()
                .unwrap()
                .clone();

            match font_pb {
                Some(ref pb) => {
                    target = pb.clone();
                    target.push(&target_dir);
                    match fs::create_dir(&target) {
                        Err(e) => {
                            if e.kind() != io::ErrorKind::AlreadyExists {
                                panic!("{:?}", e);
                            }
                        }
                        Ok(()) => (),
                    }
                    target.push(&glif_fn);
                }
                None => target.set_file_name(&glif_fn),
            }
            log::info!("Targeting {:?} to write {}", &target, &layer.name);

            let glif_struct = self.glyph.as_ref().unwrap().to_exported(layer);
            glifparser::write_to_filename(&glif_struct, &target)
                .unwrap_or_else(|e| panic!("Failed to write glif: {:?}", e));

            if font_pb.is_none() {
                log::warn!("Exported .glif without a parent UFO font. Cannot create layer(info|contents).plist.");
                if layer.color.is_some() {
                    log::error!(".glif's layer 0 calls for a color, but it has no parent UFO. Cannot create layercontents.plist, color will be lost!")
                }
                return Err(());
            }

            // In the second phase, we write the plist files layerinfo.plist and
            // layercontents.plist. We have to make sure that they either (a) don't exist or (b)
            // exist and are compatible. If incompatible, we emit a warning.
            use glifparser::glif::mfek::layer::ToLayerInfoPlist;
            // layerinfo.plist
            let needs_layerinfo = layer.color.is_some();
            if font_pb.is_none() {
                panic!("Cannot write layerinfo for glyph; not in a font!")
            }
            if needs_layerinfo {
                let mut layerinfo = target
                    .parent()
                    .expect("Cannot write layerinfo for glyph; at root of filesystem???")
                    .to_owned();
                layerinfo.push("layerinfo.plist");
                log::debug!(
                    "We are going to try to write a layerinfo.plist to {:?}",
                    &layerinfo
                );
                let mut layerinfo_p = None;
                let mut current_layerinfo_p = None;
                if Path::exists(&layerinfo) {
                    log::info!("Layer already has layerinfo, checking compatibility");
                    current_layerinfo_p = Some(plist::Value::from_file(&layerinfo).expect(
                        &format!("Failed to deserialize layerinfo.plist in {:?}", &layerinfo),
                    ));
                }

                let layerinfo_plist = layer.to_layerinfo_plist();
                if let Some(layerinfo_p2) = layerinfo_plist {
                    if let Some(cmp) = current_layerinfo_p {
                        if layerinfo_p2 != cmp {
                            log::warn!("I am replacing an existing layerinfo.plist with an incompatible layerinfo.plist. Other glyphs in this font may appear in different colors! This is not a bug in MFEKglif, you must use unique names for layers that will be differently colored *across* your font.");
                        }
                    }
                    layerinfo_p = Some(layerinfo_p2);
                };

                if let Some(li) = layerinfo_p {
                    li.to_file_xml(layerinfo).expect(&format!(
                        "Failed to write layerinfo.plist for layer {} in glyph {}",
                        i, &glif_name
                    ));
                    log::info!(
                        "Wrote layer {} of glyph {}'s layerinfo.plist. Color was {}",
                        i,
                        &glif_name,
                        layer.color.unwrap().to_string()
                    );
                }
            }
        }

        // layercontents.plist
        use glifparser::glif::mfek::layer::ToLayerContentsPlist;
        let mut our_layercontents = (&export.layers).as_slice().to_layercontents_plist();
        let layercontents = font_pb;
        if let Some(mut layercontents_f) = layercontents {
            layercontents_f.push("layercontents.plist");
            if Path::exists(&layercontents_f) {
                let current_layercontents_p =
                    Some(plist::Value::from_file(&layercontents_f).expect(&format!(
                        "Failed to deserialize layercontents.plist in {:?}",
                        &layercontents_f
                    )));
                our_layercontents = (&export.layers)
                    .as_slice()
                    .merge_layercontents_plists(current_layercontents_p.unwrap());
            }
            our_layercontents
                .to_file_xml(&layercontents_f)
                .expect(&format!(
                    "Failed to write layercontents.plist for glyph {}",
                    &glif_name
                ));
            log::info!("Wrote glyph {}'s layercontents.plist.", &glif_name);
        }
        self.begin_modification("Exported glyph");
        self.end_modification();
        Ok(())
    }

    pub fn quit(&mut self, i: &mut Interface) {
        if self.has_unsaved_changes() {
            let changes = self
                .history
                .undo_stack
                .iter()
                .rev()
                .take(10)
                .map(|he| he.description.clone())
                .collect::<Vec<_>>()
                .join(" ");
            i.push_prompt(InputPrompt::YesNo {
                question: "Unsaved changes exist in glyph. Quit anyway?".to_string(),
                afterword: format!("Recent changes:\n{}", &changes),
                func: Rc::new(move |v: &mut Editor, _, reload: bool| {
                    v.quit_requested = reload;
                    if reload {
                        log::warn!("Quit, discarding unsaved changes");
                    } else {
                        log::info!("Requested quit cancelled");
                    }
                }),
            });
        } else {
            log::info!("Quit with no unsaved changes");
            self.quit_requested = true;
        }
    }
}

pub trait ExportLayer {
    fn to_exported(&self, layer: &mut Layer<MFEKPointData>) -> Glif<MFEKPointData>;
}

/// Warning: You should always use this from MFEKglif with glif.preview.layers. If you use it with
/// the normal MFEKGlif type's layers, then you will need to apply contour operations yourself!
impl ExportLayer for MFEKGlif<MFEKPointData> {
    fn to_exported(&self, layer: &mut Layer<MFEKPointData>) -> Glif<MFEKPointData> {
        let contours: Vec<_> = layer.outline.iter_mut().map(|c| 
            c.to_cubic().cubic_mut().unwrap().clone()
        ).collect();
        let mut ret = Glif::new();
        ret.outline = Some(contours);
        ret.anchors = self.anchors.clone();
        let images = layer
            .images
            .iter()
            .map(|(img, matrix)| {
                let mut img = img.clone();
                img.set_matrix(*matrix);
                img
            })
            .collect();
        ret.images = images;
        ret.components = self.components.clone();
        ret.width = self.width;
        ret.name = self.name.clone();
        ret.unicode = self.unicode.clone();
        ret.filename = self.filename.clone();
        ret
    }
}
