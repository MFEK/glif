use crate::util;

use std::path::PathBuf;

pub fn filename_or_panic(
    filename: &Option<String>,
    filter: Option<&str>,
    start_in: Option<&str>,
) -> PathBuf {
    match filename {
        Some(file) => file.into(),
        None => match nfd::open_file_dialog(filter, start_in) {
            Ok(nfd::Response::Okay(file)) => file.into(),
            _ => util::hard_error("Exit requested"),
        },
    }
}

pub fn open_image(start_in: Option<&str>) -> Option<PathBuf> {
    match nfd::open_file_dialog(Some("jpg,png,gif,webp,bmp"), start_in) {
        Ok(nfd::Response::Okay(file)) => Some(file.into()),
        Ok(_) | Err(_) => None,
    }
}

pub fn open_filename(filter: Option<&str>, start_in: Option<&str>) -> Option<PathBuf> {
    match nfd::open_file_dialog(filter, start_in) {
        Ok(nfd::Response::Okay(file)) => Some(file.into()),
        Ok(_) | Err(_) => None,
    }
}

pub fn save_filename(filter: Option<&str>, start_in: Option<&str>) -> Option<PathBuf> {
    match nfd::open_save_dialog(filter, start_in) {
        Ok(nfd::Response::Okay(file)) => Some(file.into()),
        Ok(_) | Err(_) => None,
    }
}
