//! Global thread local state.

use glifparser::{Contour, Glif, Outline, Point};
use MFEKMath::VWSContour;

pub use crate::renderer::console::Console as RendererConsole;
use crate::renderer::Guideline;

use skulpin::winit::dpi::{PhysicalPosition, PhysicalSize};

use std::cell::RefCell;
use std::path::PathBuf;

mod tool_data;
pub use self::tool_data::*;

mod toggles;
pub use self::toggles::*;

pub struct Glyph<T> {
    pub glif: Glif<T>,
    pub filename: PathBuf,
    pub guidelines: Vec<Guideline>,
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
    pub handle_style: HandleStyle,
    pub preview_mode: PreviewMode,
    pub winsize: PhysicalSize<u32>, // for Skia
    pub factor: f32,
    pub offset: (f32, f32),
    pub dpi: f64, // from glutin scale_factor()
    pub ipc_info: Option<mfek_ipc::IPCInfo>,
    pub quit_requested: bool, // allows for quits from outside event loop, e.g. from command closures
    pub vws_previews: Option<Outline<Option<MFEKMath::piecewise::glif::PointData>>>,
    pub vws_contours: Vec<VWSContour>,
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
            preview_mode: PreviewMode::None,
            handle_style: HandleStyle::Handlebars,
            winsize: PhysicalSize {
                height: 0,
                width: 0,
            },
            factor: 1.,
            offset: (0., 0.),
            dpi: 1.,
            ipc_info: None,
            quit_requested: false,
            vws_previews: None,
            vws_contours: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointData;

thread_local!(pub static STATE: RefCell<State<Option<PointData>>> = RefCell::new(State::new()));
thread_local!(pub static CONSOLE: RefCell<RendererConsole> = RefCell::new(RendererConsole::default()));
