use glifparser::{Guideline, IntegerOrFloat};
use log;
use mfek_ipc::{self, Available, IPCInfo};

use crate::editor::Editor;
use crate::util::MFEKGlifPointData;

lazy_static::lazy_static! {
    pub static ref METADATA_STATUS: Available = mfek_ipc::module_available("metadata", "0.0-beta1").0;
}

pub fn fetch_italic(v: &mut Editor) {
    if *METADATA_STATUS == Available::No {
        return;
    }
    let filename = v.with_glyph(|glyph| glyph.filename.clone());
    let ipc_info = IPCInfo::from_glif_path("MFEKglif".to_string(), &filename.unwrap());

    let italic_angle = mfek_ipc::helpers::metadata::arbitrary(&ipc_info, &["italicAngle"]);

    if let Ok(arbdict) = italic_angle {
        if let Some(Ok(angle)) = arbdict
            .get("italicAngle")
            .map(|a| a.parse::<f32>())
        {
            v.italic_angle = angle - 90.;
        }
    } else {
        log::warn!("Failed to get italic angle. Either not in font (font not italic), or font corrupt.");
    }
}

pub fn fetch_metrics(v: &mut Editor) {
    if *METADATA_STATUS == Available::No {
        return;
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
