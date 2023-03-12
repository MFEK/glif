/// Shorthand macros for use inside editor.with() closures.
#[macro_export]
macro_rules! get_contour {
    ($v:expr, $idx:expr) => {
        $v.outline[$idx]
    };
}

#[macro_export]
macro_rules! get_contour_mut {
    ($v:expr, $idx:expr) => {
        &mut $v.outline[$idx]
    };
}

#[macro_export]
macro_rules! get_contour_len {
    ($v:expr, $idx:expr) => {
        $v.outline[$idx].len()
    };
}

#[macro_export]
macro_rules! is_contour_open {
    ($v:expr, $idx:expr) => {
        $v.outline[$idx].is_open()
    };
}

#[macro_export]
macro_rules! get_point {
    ($v:expr, $cidx:expr, $pidx:expr) => {
        $v.outline[$cidx].get_point($pidx)
    };
}

#[macro_export]
macro_rules! get_point_mut {
    ($v:expr, $cidx:expr, $pidx:expr) => {
        $v.outline[$cidx].get_point_mut($pidx)
    };
}
// This re-import is here because I think it's messy to refer to these macros using the top-level
// crate::. This allows me to have in modules e.g. `use crate::editor::macros::get_point`, which is
// our preferred way of importing them.
pub use {
    get_contour, get_contour_len, get_contour_mut, get_point, get_point_mut, is_contour_open,
};
