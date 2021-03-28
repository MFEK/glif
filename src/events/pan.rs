// Pan
use super::prelude::*;

pub fn mouse_moved<T>(position: (f64, f64), v: &RefCell<state::State<T>>) -> bool {
    let old_mposition = v.borrow().absolute_mousepos.clone();
    let mut offset = v.borrow().offset;
    let mposition = update_mousepos(position, &v, true);
    if !v.borrow().mousedown {
        return false;
    }
    offset.0 += (mposition.0 - old_mposition.0).floor() as f32;
    offset.1 += (mposition.1 - old_mposition.1).floor() as f32;
    //offset = (mposition.x as f32, mposition.y as f32);
    v.borrow_mut().offset = offset;
    update_viewport(None, None, &v);
    true
}
