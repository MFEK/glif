use directories::BaseDirs;
use lazy_static::lazy_static;
use log;

use std::path::PathBuf;

lazy_static! {
    pub static ref CONFIG_PATH: PathBuf = {
        let mut ret = BaseDirs::new().unwrap().config_dir().to_path_buf();
        ret.push("MFEK");
        ret.push("glif");
        log::info!("Configuration directory is {:?}", &ret);
        ret
    };
}
