use glifparser::{Guideline, IntegerOrFloat};
use log;
use mfek_ipc::{self, Available, IPCInfo};

use crate::editor::Editor;

pub fn fetch_metrics(v: &mut Editor) {
    let (status, _) = mfek_ipc::module_available("metadata");
    if status != Available::Yes {
        return;
    }
    let filename = v.with_glyph(|glyph| glyph.filename.clone());
    let ipc_info = IPCInfo::from_glif_path("MFEKglif".to_string(), &filename.unwrap());

    let metrics = mfek_ipc::helpers::metadata::ascender_descender(&ipc_info);

    if let Ok(metrics) = metrics {
        let names = &["ascender", "descender"];
        for (i, metric) in [metrics.0, metrics.1].into_iter().enumerate() {
            let guideline = Guideline::from_x_y_angle(0., metric, IntegerOrFloat::default()).name(names[i].to_string());
            log::debug!("Adding metrics guideline: {:?}", &guideline);
            v.guidelines.push(guideline);
        }
    } else {
        log::error!("Failed to get ascender/descender. Not in font, or font corrupt.");
    }

    let guidelines = mfek_ipc::helpers::metadata::guidelines(&ipc_info);
    if let Ok(guidelines) = guidelines {
        v.guidelines.extend(guidelines);
    } else {
        log::error!("Failed to get font-level guidelines. Not in font, or font corrupt.");
    }

    v.ipc_info = Some(ipc_info);
}
