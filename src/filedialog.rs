use std::path::PathBuf;

extern crate nfd;

pub fn filename_or_panic(filename: &Option<String>, filter: Option<&str>, start_in: Option<&str>) -> PathBuf {
    match filename {
        Some(file) => file.into(),
        None => match nfd::open_file_dialog(filter, start_in) {
            Ok(nfd::Response::Okay(file)) => file.into(),
            _ => panic!("Exit requested"),
        },
    }
}
