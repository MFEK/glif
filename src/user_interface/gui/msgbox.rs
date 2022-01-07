macro_rules! gui_error {
    ($($format_args:tt),*) => {
        use log;
        use msgbox::{self, IconType::*};
        use std::thread;
        log::error!($($format_args),*);
        thread::spawn(|| {
            match msgbox::create("MFEKglif critical error", &format!($($format_args),*), Error) {
                Ok(_) => log::trace!("Opened crash msgbox successfully"),
                Err(e) => log::error!("Failed to create error box! {:?}", e),
            }
        });
    }
}
pub(crate) use gui_error;
