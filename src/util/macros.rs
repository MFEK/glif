// $v of type RefCell<State<T>> (the global state)
// Returns type glifparser::Glif
/*
#[macro_export]
macro_rules! get_outline_mut {
    ($v:ident) => {
        $v.borrow_mut()
            .glyph
            .as_mut()
            .unwrap()
            .glif
            .outline
            .as_mut()
            .unwrap()
    };
}

#[macro_export]
macro_rules! get_outline {
    ($v:ident) => {
        $v.borrow()
            .glyph
            .as_ref()
            .unwrap()
            .glif
            .outline
            .as_ref()
            .unwrap()
    };
}
*/

#[macro_export]
macro_rules! get_outline {
    ($v:ident) => {
        $v.outline.as_ref().unwrap()
    };
}

#[macro_export]
macro_rules! get_outline_mut {
    ($v:ident) => {
        $v.outline.as_mut().unwrap()
    };
}

#[macro_export]
macro_rules! get_contour {
    ($v:ident, $idx:expr) => {
        get_outline!($v)[$idx]
    };
}

#[macro_export]
macro_rules! get_contour_mut {
    ($v:ident, $idx:expr) => {
        &mut $v.outline.as_mut().unwrap()[$idx]
    };
}

#[macro_export]
macro_rules! get_contour_len {
    ($v:ident, $idx:expr) => {
        $v.outline.as_ref().unwrap()[$idx].len()
    };
}

#[macro_export]
macro_rules! get_contour_type {
    ($v:ident, $idx:expr) => {
        $v.outline.as_ref().unwrap()[$idx].first().unwrap().ptype
    };
}