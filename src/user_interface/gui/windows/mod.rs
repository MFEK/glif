use std::{default, str::FromStr};

use egui::Ui;

use super::textedit_buffer::EditBuffer;

pub mod inspection_window;
pub mod grid_window;
pub mod tool_window;
pub mod layer_list;

pub fn egui_parsed_textfield<D>(ui: &mut Ui, id: impl Into<String>, default: D, editbuf: &mut impl EditBuffer) -> D
    where D: FromStr + ToString {
    let id: &String = &id.into();
    let prev_buf = editbuf.get_buf(id, &default.to_string()).clone();
    let response = ui.text_edit_singleline(editbuf.get_buf(id, &default.to_string()));

    if let Err(_) = editbuf.get_buf(id, &default.to_string()).parse::<D>() {
        if !editbuf.get_buf(id, &default.to_string()).is_empty() {
            editbuf.set_buf(id, prev_buf);
        }
    }

    if response.lost_focus() {
        if let Ok(fp) = editbuf.get_buf(id, &default.to_string()).parse::<D>() {
            return fp;
        }
    }

    if !response.has_focus() {
        editbuf.reset(id);
    }

    default
}