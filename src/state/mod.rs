//! Global thread local state.

use glifparser::{Contour, Glif, Layer, Point};

pub use crate::renderer::console::Console as RendererConsole;
use crate::{events::{EditorEvent, Tool, ToolEnum, pan::Pan, tool_enum_to_tool}, renderer::Guideline};

use std::{borrow::{Borrow, BorrowMut}, cell::RefCell, collections::HashMap};
use std::path::PathBuf;

mod tool_data;
use self::history::HistoryType;
pub use self::tool_data::*;

mod toggles;
pub use self::toggles::*;

mod history;
pub use self::history::HistoryEntry;

pub struct Glyph<P: glifparser::PointData> {
    pub glif: Glif<P>,
    pub filename: PathBuf,
    pub guidelines: Vec<Guideline>,
}

// Thread local state.
pub struct Editor {
    glyph: Option<Glyph<PointData>>,
    modifying: bool, // is the active layer being modified?
    history: Vec<HistoryEntry<PointData>>,
    active_tool: Box<dyn Tool>,
    active_tool_enum: ToolEnum,
    
    pub selected: Vec<Point<PointData>>,
    pub mousedown: bool,
    pub mousepos: (f64, f64),
    pub absolute_mousepos: (f64, f64),

    pub point_labels: PointLabels,
    pub handle_style: HandleStyle,
    pub preview_mode: PreviewMode,
    pub winsize: (u32, u32), // for Skia
    pub factor: f32,
    pub offset: (f32, f32),
    pub dpi: f64, // from glutin scale_factor()
    pub ipc_info: Option<mfek_ipc::IPCInfo>,
    pub quit_requested: bool, // allows for quits from outside event loop, e.g. from command closures

    pub contour_idx: Option<usize>,   // index into Outline
    pub point_idx: Option<usize>, 
    pub layer_idx: Option<usize>, // active layer

    pub previews: HashMap<usize, HashMap<usize, Contour<PointData>>>,
}

impl Editor {
    pub fn new() -> Editor {
        // FIXME: Making a new one doesn't get current mouse position nor window size.
        Editor {
            glyph: None,

            active_tool: Box::new(Pan::new()),
            active_tool_enum: ToolEnum::Pan,

            history: Vec::new(),
            previews: HashMap::new(),

            // TODO: refactor these out of State
            mousedown: false,
            mousepos: (0., 0.),
            absolute_mousepos: (0., 0.),

            point_labels: PointLabels::None,
            preview_mode: PreviewMode::None,
            handle_style: HandleStyle::Handlebars,

            winsize: (0, 0),
            factor: 1.,
            offset: (0., 0.),
            dpi: 1.,
            ipc_info: None,
            quit_requested: false,

            // selection state
            layer_idx: None,
            contour_idx: None,   // index into Outline
            point_idx: None, // index into Contour
            selected: Vec::new(),
            modifying: false,
        }
    }

    pub fn set_glyph(&mut self, glyph: Glyph<PointData>)
    {
        self.glyph = Some(glyph);
        self.layer_idx = Some(0);
    }
    // this function MUST be called before calling with_active_layer_mut or that function will panic
    // this pushes a clone of the current layer onto the history stack and invokes the appropriate contour op
    // events like rebuilding previews
    pub fn begin_layer_modification(&mut self, description: &str) {
        if self.modifying == true { panic!("Began a new modification with one in progress!")}

        self.history.push(HistoryEntry {
            description: description.to_owned(),
            layer_idx: self.layer_idx,
            contour_idx: self.contour_idx,
            point_idx: self.point_idx,
            layer: self.glyph.as_ref().unwrap().glif.layers[self.layer_idx.unwrap()].clone(),
            kind: HistoryType::LayerModified
        });

        self.modifying = true;
    }

    // This should ideally be the only way tools are modifying the glyph. You call this and then do your edits inside the closure.
    // This is to prevent history-less modifications from occuring.
    pub fn with_active_layer_mut<F, R>(&mut self, mut closure: F) -> R
        where F: FnMut(&mut Layer<PointData>) -> R {
        if self.modifying == false { panic!("A modification is not in progress!")}
        let glyph = self.glyph.as_mut().unwrap();
        closure(&mut glyph.glif.layers[self.layer_idx.unwrap()])
    }

    pub fn with_active_layer<F, R>(&self, mut closure: F) -> R
        where F: FnMut(&Layer<PointData>) -> R {
        closure(&self.glyph.as_ref().unwrap().glif.layers[self.layer_idx.unwrap()])
    }


    pub fn with_glif<F, R>(&self, mut closure: F) -> R 
        where F: FnMut(&Glif<PointData>) -> R {
        closure(&self.glyph.as_ref().unwrap().glif)
    }

    pub fn with_glyph<F, R>(&self, mut closure: F) -> R
        where F: FnMut(&Glyph<PointData>) -> R {
        closure(&self.glyph.as_ref().unwrap())
    }

    // Please do not call this. It's currently used by externals that probably need to be
    // TODO: moved into this or a member struct of this
    pub fn with_glyph_mut<F, R>(&mut self, mut closure: F) -> R 
        where F: FnMut(&mut Glyph<PointData>) -> R 
    {
        closure(&mut self.glyph.as_mut().unwrap())
    }
    // this function finalizes an edit by invoking the correct events and setting layer_modify to false
    pub fn end_layer_modification(&mut self) {
        if self.modifying == false { return };

        // TODO: Events here.
        self.modifying = false;
    }

    pub fn set_tool(&mut self, tool: ToolEnum ) {
        if self.active_tool_enum == tool { return };

        self.end_layer_modification();
        self.active_tool_enum = tool;
        self.active_tool = tool_enum_to_tool(tool);
    }

    pub fn get_tool(&self) -> ToolEnum {
        self.active_tool_enum
    }

    pub fn get_tool_mut(&mut self) -> &mut Box<dyn Tool>
    {
        &mut self.active_tool
    }

    pub fn dispatch_editor_event(&mut self, event: EditorEvent) {
        let mut active_tool = dyn_clone::clone_box(&*self.active_tool);
        active_tool.handle_event(self, event);
        self.active_tool = active_tool;
    }

    pub fn undo(&mut self) {
        let entry = self.history.pop();
        
        if let Some(undo_entry) = entry {
            self.glyph.as_mut().unwrap().glif.layers[undo_entry.layer_idx.unwrap()] = undo_entry.layer;
            self.layer_idx = undo_entry.layer_idx;
            self.contour_idx = undo_entry.contour_idx;
            self.point_idx = undo_entry.point_idx;
        }
    }

    pub fn redo() {
        // TODO:
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointData;
impl glifparser::PointData for PointData {}

thread_local!(pub static CONSOLE: RefCell<RendererConsole> = RefCell::new(RendererConsole::default()));
