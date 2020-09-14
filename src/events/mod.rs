pub mod prelude;
use self::prelude::*;

pub mod console;
pub mod pan;
pub mod pen;
pub mod select;
pub mod zoom;

pub use self::zoom::{zoom_in_factor, zoom_out_factor};

// Generic events
pub fn center_cursor(winit_window: &Window) -> Result<(), winit::error::ExternalError> {
    let mut center = winit_window.outer_size();
    center.width /= 2;
    center.height /= 2;
    STATE.with(|v| {
        v.borrow_mut().absolute_mousepos = PhysicalPosition::from((center.width, center.height))
    });
    winit_window.set_cursor_position(winit::dpi::PhysicalPosition::new(
        center.width as i32,
        center.height as i32,
    ))
}

pub fn update_viewport<T>(
    offset: Option<(f32, f32)>,
    scale: Option<f32>,
    v: &RefCell<state::State<T>>,
) {
    let offset = match offset {
        None => v.borrow().offset,
        Some(offset) => offset,
    };
    let scale = match scale {
        None => v.borrow().factor,
        Some(scale) => scale,
    };
    v.borrow_mut().factor = scale;
    v.borrow_mut().offset = offset;
}

pub fn update_mousepos<T>(
    position: PhysicalPosition<f64>,
    v: &RefCell<state::State<T>>,
    pan: bool,
) -> PhysicalPosition<f64> {
    let factor = 1. / v.borrow().factor as f64;
    let uoffset = v.borrow().offset;
    let offset = (uoffset.0 as f64, uoffset.1 as f64);

    let absolute_mposition = PhysicalPosition::from(((position.x).floor(), (position.y).floor()));
    let mposition = PhysicalPosition::from((
        ((position.x).floor() - offset.0) * factor,
        ((position.y).floor() - offset.1) * factor,
    ));

    v.borrow_mut().absolute_mousepos = absolute_mposition;
    v.borrow_mut().mousepos = mposition;
    if pan {
        absolute_mposition
    } else {
        mposition
    }
}

pub fn mode_switched(from: Mode, to: Mode) {
    assert!(from != to);
    PEN_DATA.with(|v| v.borrow_mut().contour = None);
}

#[macro_export]
///! Given a field on the State struct, and an enumerator that implements IntoEnumIterator, cycle
///! through its variants and update state. An optional condition is provided. $state is expected to
///! be an inner thread::LocalKey<State>.
macro_rules! trigger_toggle_on {
    ($state:ident, $state_var:ident, $enum:ident, $cond:expr) => {
        let $state_var = $state.borrow().$state_var;
        if $cond {
            let mut e = $enum::into_enum_iter()
                .cycle()
                .skip(1 + $state_var as usize);
            let n = e.next().unwrap();
            $state.borrow_mut().$state_var = n;
        }
    };
    ($state:ident, $state_var:ident, $enum:ident) => {
        trigger_toggle_on!($state, $state_var, $enum, true);
    }
}
