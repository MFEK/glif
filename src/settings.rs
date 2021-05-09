use app_dirs::{AppDataType, AppInfo, app_dir};
use lazy_static::lazy_static;

use std::path::PathBuf;

pub const APP_INFO: AppInfo = AppInfo {
    name: "MFEK",
    author: "MFEK team",
};

lazy_static! {
    pub static ref CONFIG_PATH: PathBuf = app_dir(AppDataType::UserConfig, &APP_INFO, "glif").unwrap();
}
