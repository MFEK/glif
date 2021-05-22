use crate::editor::Editor;
use std::{cell::RefCell, collections::HashMap};

type Callback = Box<(dyn Fn(&mut Editor, Vec<String>) -> () + 'static)>;

fn callback<F>(f: F) -> Callback
where
    F: Fn(&mut Editor, Vec<String>) -> () + 'static,
{
    Box::new(f) as Callback
}

pub fn initialize_console_commands() {
    MAP.with(|h| {
        h.borrow_mut().insert("vpoffset", ("Set viewport origin", callback(|v, s| {
            if s.len() != 2 { return; } // FIXME: Tell user about errors!
            if let (Ok(ox), Ok(oy)) = (s[0].parse(), s[1].parse()) {
                v.update_viewport(Some((ox, oy)), None);
            }
        })));
    
        h.borrow_mut().insert("vpfactor", ("Set viewport zoom factor", callback(|v, s| {
            if s.len() != 1 { return; } // FIXME: Tell user about errors!
            if let Ok(factor) = s[0].parse() {
                v.update_viewport(None, Some(factor));
            }
        })));
    
        h.borrow_mut().insert("q", ("Quit", callback(|v, _s| {
            v.quit_requested = true;
        })));
    })
}

thread_local! {
    pub static MAP: RefCell<HashMap<&'static str, (&'static str, Callback)>> = RefCell::new(HashMap::new());
}
