use glifparser::{Guideline, IntegerOrFloat};
use log;
use mfek_ipc::{self, module, IPCInfo};

use crate::editor::{events::*, Editor};
use crate::user_interface::Interface;
use crate::util::MFEKGlifPointData;

use std::path::PathBuf;
use std::process;

lazy_static::lazy_static! {
    pub static ref METADATA_AVAILABLE: Result<(module::Version<'static>, PathBuf), ()> = mfek_ipc::module::available("metadata", "0.0.2-beta1");
}

pub fn header() {
    mfek_ipc::display_elaborate_header(
        "glif",
        env!("MFEK_VERSION"),
        Some(env!("MFEK_COMPILED_AT").parse().unwrap()),
    );
}

impl Editor {
    pub fn write_metrics(&mut self, interface: &mut Interface) {
        let exe = match &*METADATA_AVAILABLE {
            Ok((_, exe)) => exe,
            Err(e) => return log::error!("Failed to launch MFEKmetadata! {:?}", e), // ()
        };

        let filename = self.with_glyph(|glyph| glyph.filename.clone()).unwrap();
        let ipc_info = IPCInfo::from_glif_path("MFEKglif".to_string(), &filename);

        let font = if let Some(font) = ipc_info.font {
            font
        } else {
            log::error!("Requested a write of metrics when not in a font, global metrics/guidelines can't be written.");
            return;
        };

        let mut args = vec![];
        let mut guidelines = vec![];
        let mut has_ascender = false;
        let mut has_descender = false;
        for g in self.guidelines.iter() {
            match g.name.as_ref().map(|n| n.as_str()) {
                Some("ascender") => {
                    if !has_ascender {
                        args.extend(["-k".to_string(), "ascender".to_string(), "-v".to_string()]);
                        args.push(format!("<real>{}</real>", g.at.y));
                        has_ascender = true;
                    } else {
                        continue;
                    }
                }
                Some("descender") => {
                    if !has_descender {
                        args.extend(["-k".to_string(), "descender".to_string(), "-v".to_string()]);
                        args.push(format!("<real>{}</real>", g.at.y));
                        has_descender = true;
                    } else {
                        continue;
                    }
                }
                _ => {}
            }
            if !g.data.as_guideline().format {
                guidelines.push(g);
            }
        }
        let guidelinesp = plist::Value::Array(
            guidelines
                .iter()
                .map(|g| plist::Value::Dictionary(g.as_plist_dict()))
                .collect::<Vec<_>>(),
        );
        let mut guidelinesv = vec![];
        plist::to_writer_xml(&mut guidelinesv, &guidelinesp).unwrap();
        args.extend([
            "-k".to_string(),
            "guidelines".to_string(),
            "-v".to_string(),
            String::from_utf8(guidelinesv).unwrap(),
        ]);
        let ok = process::Command::new(&exe)
            .arg(&font)
            .arg("arbitrary")
            .args(&args)
            .status();
        if let Err(e) = ok {
            log::error!("Failed to execute MFEKmetadata to rewrite metrics! {:?}", e);
        } else {
            self.dispatch_editor_event(
                interface,
                EditorEvent::IOEvent {
                    event_type: IOEventType::FontinfoWritten,
                    path: filename.clone(),
                },
            );
        }
    }
}

pub fn fetch_italic(v: &mut Editor) {
    if let Err(_) = &*METADATA_AVAILABLE {
        return log::debug!("Not trying fetch_italic, MFEKmetadata unavailable");
    }

    let filename = v.with_glyph(|glyph| glyph.filename.clone());
    let ipc_info = IPCInfo::from_glif_path("MFEKglif".to_string(), &filename.unwrap());

    let italic_angle = mfek_ipc::helpers::metadata::arbitrary(&ipc_info, &["italicAngle"]);

    if let Ok(arbdict) = italic_angle {
        if let Some(Ok(angle)) = arbdict.get("italicAngle").map(|a| a.parse::<f32>()) {
            v.italic_angle = angle - 90.;
        }
    } else {
        log::warn!(
            "Failed to get italic angle. Either not in font (font not italic), or font corrupt."
        );
    }
}

pub fn fetch_metrics(v: &mut Editor) {
    if let Err(_) = &*METADATA_AVAILABLE {
        return log::debug!("Not trying fetch_italic, MFEKmetadata unavailable");
    }

    let filename = v.with_glyph(|glyph| glyph.filename.clone());
    let ipc_info = IPCInfo::from_glif_path("MFEKglif".to_string(), &filename.unwrap());

    let metrics = mfek_ipc::helpers::metadata::ascender_descender(&ipc_info);

    if let Ok(metrics) = metrics {
        const NAMES: &[&str] = &["ascender", "descender"];
        for (i, metric) in [metrics.0, metrics.1].into_iter().enumerate() {
            let (fixed, format, right) = (false, true, false);
            let guideline = Guideline::from_x_y_angle(0., metric, IntegerOrFloat::default())
                .name(NAMES[i].to_string())
                .data(MFEKGlifPointData::new_guideline_data(fixed, format, right));
            log::trace!("Adding metrics guideline: {:?}", &guideline);
            v.guidelines.push(guideline);
        }
    } else {
        log::warn!("Failed to get ascender/descender. Not in font, or font corrupt.");
    }

    let guidelines = mfek_ipc::helpers::metadata::guidelines(&ipc_info);

    if let Ok(guidelines) = guidelines {
        v.guidelines.extend(guidelines);
    } else {
        log::warn!("Failed to get font-level guidelines. Not in font, or font corrupt.");
    }

    v.ipc_info = Some(ipc_info);
}

pub fn launch_fs_watcher(v: &mut Editor) {
    let filename = v.with_glyph(|glyph| glyph.filename.clone());
    let ipc_info = IPCInfo::from_glif_path("MFEKglif".to_string(), &filename.as_ref().unwrap());
    if let Some(font) = ipc_info.font {
        mfek_ipc::notifythread::launch(font, v.filesystem_watch_tx.clone());
    } else {
        mfek_ipc::notifythread::launch(
            ipc_info.glyph.unwrap().parent().unwrap().to_owned(),
            v.filesystem_watch_tx.clone(),
        );
    }
}
