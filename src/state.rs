//! Global thread local state.

use glifparser;
use imgui;

use enum_iterator::IntoEnumIterator;
use glifparser::{Contour, Glif, Point};
use mfeq_ipc::IPCInfo;
use renderer::Guideline;
use skulpin::skia_safe::Surface;
use skulpin::winit::dpi::{PhysicalPosition, PhysicalSize};

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

pub struct Glyph<T> {
    pub glif: Glif<T>,
    pub filename: PathBuf,
    pub guidelines: Vec<Guideline>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Pan,
    Pen,
    Select,
    Zoom,
}

#[derive(IntoEnumIterator, Debug, Clone, Copy, PartialEq)]
pub enum PointLabels {
    None,
    Numbered,
    Locations,
}

pub struct PenData {
    pub contour: Option<usize>,   // index into Outline
    pub cur_point: Option<usize>, // index into Contour
}

impl PenData {
    fn new() -> Self {
        PenData {
            contour: None,
            cur_point: None,
        }
    }
}

// Thread local state.
pub struct State<T> {
    pub mode: Mode,
    pub glyph: Option<Glyph<T>>,
    pub selected: Vec<Point<T>>,
    pub mousedown: bool,
    pub mousepos: PhysicalPosition<f64>,
    pub absolute_mousepos: PhysicalPosition<f64>,
    pub corner_one: Option<PhysicalPosition<f64>>,
    pub corner_two: Option<PhysicalPosition<f64>>,
    // Whether to show the selection box on screen
    pub show_sel_box: bool,
    pub point_labels: PointLabels,
    pub winsize: PhysicalSize<u32>, // for Skia
    pub factor: f32,
    pub offset: (f32, f32),
    pub dpi: f64, // from glutin scale_factor()
    pub ipc_info: Option<mfeq_ipc::IPCInfo>,
}

impl<T> State<T> {
    pub fn new() -> State<T> {
        // FIXME: Making a new one doesn't get current mouse position nor window size.
        State {
            glyph: None,
            mode: Mode::Select,
            selected: Vec::new(),
            mousedown: false,
            mousepos: PhysicalPosition { x: 0., y: 0. },
            absolute_mousepos: PhysicalPosition { x: 0., y: 0. },
            corner_one: None,
            corner_two: None,
            show_sel_box: false,
            point_labels: PointLabels::None,
            winsize: PhysicalSize {
                height: 0,
                width: 0,
            },
            factor: 1.,
            offset: (0., 0.),
            dpi: 1.,
            ipc_info: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointData;

thread_local!(pub static STATE: RefCell<State<Option<PointData>>> = RefCell::new(State::new()));
thread_local!(pub static PEN_DATA: RefCell<PenData> = RefCell::new(PenData::new()));
