macro_rules! gui_error {
    ($($format_args:expr),*) => {
        {
            use log;
            log::error!($($format_args),*);
            let err = format!($($format_args),*);
            use msgbox::{self, IconType::Error};
            match msgbox::create("MFEKglif critical error", &err, Error) {
                Ok(_) => log::trace!("Opened crash msgbox successfully"),
                Err(msgbox_e) => log::error!("Failed to create error box! {:?}", msgbox_e),
            }
        }
    }
}
pub(crate) use gui_error;
