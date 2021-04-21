/// Shorthand macros for use inside v.with() closures.
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

#[macro_export]
///! Given a field on the State struct, and an enumerator that implements IntoEnumIterator, cycle
///! through its variants and update state. An optional condition is provided. $state is expected to
///! be an inner thread::LocalKey<State>.
macro_rules! trigger_toggle_on {
    ($state:ident, $state_var:ident, $enum:ident, $cond:expr) => {
        let $state_var = $state.viewport.$state_var;
        if $cond {
            let mut e = $enum::into_enum_iter()
                .cycle()
                .skip(1 + $state_var as usize);
            let n = e.next().unwrap();
            $state.viewport.$state_var = n;
        }
    };
    ($state:ident, $state_var:ident, $enum:ident) => {
        trigger_toggle_on!($state, $state_var, $enum, true);
    };
}
