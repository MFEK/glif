/// Shorthand macros for use inside editor.with() closures.
#[macro_export]
macro_rules! get_contour {
    ($v:ident, $idx:expr) => {
        $v.outline[$idx].inner
    };
}

#[macro_export]
macro_rules! get_contour_mut {
    ($v:ident, $idx:expr) => {
        &mut $v.outline[$idx].inner
    };
}

#[macro_export]
macro_rules! get_contour_len {
    ($v:ident, $idx:expr) => {
        $v.outline[$idx].inner.len()
    };
}

#[macro_export]
macro_rules! get_contour_type {
    ($v:ident, $idx:expr) => {
        $v.outline[$idx].inner.first().unwrap().ptype
    };
}

#[macro_export]
macro_rules! get_point {
    ($v:ident, $cidx:expr, $pidx:expr) => {
        $v.outline[$cidx].inner[$pidx]
    };
}