use glifparser::{Guideline, IntegerOrFloat::{self, Float}};
use log;
use mfek_ipc::{self, Available, IPCInfo};
use serde_json;

use crate::editor::Editor;

use std::{process, str};

pub fn fetch_metrics(v: &mut Editor) {
    let (status, kmdbin) = mfek_ipc::module_available("metadata");
    if status != Available::Yes {
        return;
    }
    let filename = v.with_glyph(|glyph| glyph.filename.clone());
    let ipc_info = IPCInfo::from_glif_path("MFEKglif".to_string(), &filename.unwrap());

    match &ipc_info.font.as_ref() {
        Some(ref font) => {
            let command = process::Command::new(&kmdbin)
                .arg(font)
                .args(&["arbitrary", "-k", "ascender", "-k", "descender"])
                .output()
                .expect("No output, font corrupt?");

            let lines_vec = str::from_utf8(&command.stdout).unwrap();
            let mut lines_iter = lines_vec.lines();
            let nlines = lines_iter.count();
            lines_iter = lines_vec.lines();

            if nlines != 2 {
                log::error!("Cannot set ascender/descender, font corrupt?");
            } else {
                let names = &["ascender", "descender"];
                for (i, line) in lines_iter.enumerate() {
                    let y = line.parse().expect("Font is corrupt, metrics not numeric!");
                    let guideline = Guideline::from_name_x_y_angle(names[i].to_string(), 0., y, IntegerOrFloat::default());
                    log::debug!("Adding guideline: {:?}", &guideline);
                    v.guidelines.push(guideline);
                }
            }

            let command = process::Command::new(&kmdbin)
                .arg(font)
                .args(&["arbitrary", "-k", "guidelines"])
                .output();

            command.map(|output| {
                let lines_vec = str::from_utf8(&output.stdout).unwrap();
                let mut lines_iter = lines_vec.lines();
                let line: Vec<std::collections::BTreeMap<&str, serde_json::Value>> = lines_iter.next().map(|line| {
                    log::trace!("{}", &line);
                    let v = serde_json::from_str(line);
                    if let Ok(vv) = v {
                        vv
                    } else {
                        log::error!("{:?}", v);
                        vec![]
                    }
                }).unwrap_or(vec![]);
                log::trace!("{:?}", &line);
                for (i, guideline) in line.iter().enumerate() {
                    let (name_o, x_o, y_o, angle_o) = (guideline.get("name"), guideline.get("x"), guideline.get("y"), guideline.get("angle"));
                    if let (Some(name_v), Some(x_v), Some(y_v), Some(angle_v)) = (name_o, x_o, y_o, angle_o) {
                        match (name_v.as_str(), x_v.as_f64(), y_v.as_f64(), angle_v.as_f64()) {
                            (Some(name), Some(x), Some(y), Some(angle)) => {
                                v.guidelines.push(Guideline::from_name_x_y_angle(name.to_string(), x as f32, y as f32, Float(angle as f32)));
                            },
                            (None, Some(x), Some(y), Some(angle)) => {
                                v.guidelines.push(Guideline::from_name_x_y_angle(format!("Unnamed @{}", i), x as f32, y as f32, Float(angle as f32)));
                            }
                            _ => {}
                        }
                    }
                    log::debug!("Adding guideline: {:?}", &guideline);
                }
            }).unwrap_or(());
        }
        None => {
            log::error!("Cannot set metrics, .glif file not part of a UFO!");
        }
    }

    v.ipc_info = Some(ipc_info);
}
