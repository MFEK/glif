use super::glifparser;
use glifparser::{Glif, Point};
use glutin::dpi::{PhysicalPosition, PhysicalSize};
use reclutch::skia::Surface;
use std::cell::RefCell;

pub struct Glyph {
    pub glif: Glif,
    pub filename: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Move,
    Select,
}

// Thread local state.
pub struct State {
    pub mode: Mode,
    pub glyph: Option<Glyph>,
    pub selected: Vec<Point>,
    pub mousedown: bool,
    pub mousepos: PhysicalPosition<f64>,
    pub corner_one: Option<PhysicalPosition<f64>>,
    pub corner_two: Option<PhysicalPosition<f64>>,
    // Whether pub to show the selection box on screen
    pub show_sel_box: bool,
    pub winsize: PhysicalSize<u32>, // for Skia
    pub factor: f32,
    pub offset: (f32, f32),
    pub dpi: f64, // from glutin scale_factor()
}

impl State {
    pub fn new() -> State {
        // FIXME: Making a new one doesn't get current mouse position nor window size.
        State {
            glyph: None,
            mode: Mode::Select,
            selected: Vec::new(),
            mousedown: false,
            mousepos: PhysicalPosition { x: 0., y: 0. },
            corner_one: None,
            corner_two: None,
            show_sel_box: false,
            winsize: PhysicalSize {
                height: 0,
                width: 0,
            },
            factor: 1.,
            offset: (0., 0.),
            dpi: 1.,
        }
    }
}

thread_local!(pub static state: RefCell<State> = RefCell::new(State::new()));
