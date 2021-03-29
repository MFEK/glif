use crate::events;
use std::collections::HashMap;

type Callback = Box<(dyn Fn(Vec<String>) -> () + 'static)>;

fn callback<F>(f: F) -> Callback
where
    F: Fn(Vec<String>) -> () + 'static,
{
    Box::new(f) as Callback
}

use crate::STATE;
thread_local! {
    pub static MAP: HashMap<&'static str, (&'static str, Callback)> = {
        let mut h = HashMap::new();

        h.insert("vpoffset", ("Set viewport origin", callback(|s| {
            if s.len() != 2 { return; } // FIXME: Tell user about errors!
            if let (Ok(ox), Ok(oy)) = (s[0].parse(), s[1].parse()) {
                STATE.with(|v| {
                    v.borrow_mut().offset = (ox, oy);
                    events::update_viewport(Some((ox, oy)), None, &v);
                });
            }
        })));

        h.insert("vpfactor", ("Set viewport zoom factor", callback(|s| {
            if s.len() != 1 { return; } // FIXME: Tell user about errors!
            if let Ok(factor) = s[0].parse() {
                STATE.with(|v| {
                    v.borrow_mut().factor = factor;
                    events::update_viewport(None, Some(factor), &v);
                });
            }
        })));

        h.insert("q", ("Quit", callback(|_| {
            STATE.with(|v| v.borrow_mut().quit_requested = true);
        })));

        h
    };
}
