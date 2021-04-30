use glifparser::{Contour, Guideline, MFEKGlif, glif::{HistoryEntry, HistoryType, Layer, MFEKPointData}};

pub use crate::renderer::console::Console as RendererConsole;
use crate::tools::{EditorEvent, Tool, ToolEnum, pan::Pan, tool_enum_to_tool};

pub use skulpin::skia_safe::Contains as _;
pub use skulpin::skia_safe::{Canvas, Matrix, Path as SkPath, Point as SkPoint, Rect as SkRect};
pub use crate::renderer::points::calc::*;

use std::cell::RefCell;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::get_contour_mut;
use crate::get_outline_mut;

mod toggles;
pub use self::toggles::*;

pub mod util;

pub mod viewport;
pub use self::viewport::Viewport;

pub mod input;
pub use self::input::MouseInfo;

pub mod selection;
pub mod layers;

pub mod history;
use crate::editor::history::History;

pub mod operations;

#[macro_use]
pub mod macros;

/// This is the main object that holds the state of the editor.
pub struct Editor {
    glyph: Option<MFEKGlif<MFEKPointData>>,
    modifying: bool, // is the active layer being modified?
    history: History,
    active_tool: Box<dyn Tool>,
    active_tool_enum: ToolEnum,
    clipboard: Option<Layer<MFEKPointData>>,

    layer_idx: Option<usize>, // active layer
    pub preview: Option<MFEKGlif<MFEKPointData>>,
    pub contour_idx: Option<usize>,   // index into Outline
    pub point_idx: Option<usize>, 
    pub selected: HashSet<(usize, usize)>,
 
    pub mouse_info: MouseInfo,
    pub viewport: Viewport,

    pub ipc_info: Option<mfek_ipc::IPCInfo>,
    pub quit_requested: bool, // allows for quits from outside event loop, e.g. from command closures
}

impl Editor {
    pub fn new() -> Editor {
        // FIXME: Making a new one doesn't get current mouse position nor window size.
        Editor {
            glyph: None,
            preview: None,

            active_tool: Box::new(Pan::new()),
            active_tool_enum: ToolEnum::Pan,

            history: History::new(),
            // TODO: refactor these out of State

            // TODO: Add a default for this.
            mouse_info: MouseInfo::default(),
            viewport: Viewport::default(),

            ipc_info: None,
            quit_requested: false,

            // selection state
            layer_idx: None,
            contour_idx: None,   // index into Outline
            point_idx: None, // index into Contour
            selected: HashSet::new(),
            clipboard: None,

            modifying: false,
        }
    }
    
    
    pub fn set_glyph(&mut self, glyph: MFEKGlif<MFEKPointData>)
    {
        self.glyph = Some(glyph);
        self.layer_idx = Some(0);
        self.rebuild_previews();
    }

    /// This is the function that powers the editor. Tools recieve events from the Editor and then use them to modify state.
    /// Adding new events is as simple as creating a new anonymous struct to EditorEvent and a call to this function in the appropriate
    /// place.Tools can then implement behavior for that event in their handle_event implementation.
    pub fn dispatch_editor_event(&mut self, event: EditorEvent) {
        let mut active_tool = dyn_clone::clone_box(&*self.active_tool);
        active_tool.handle_event(self, event);
        self.active_tool = active_tool;
    }

    /// This function MUST be called before calling with_active_<layer/glif>_mut or it will panic.
    /// Pushes a clone of the current layer onto the history stack and puts the editor in a modifying state.
    pub fn begin_layer_modification(&mut self, description: &str) {
        self.history.redo_stack = vec!();
        if self.modifying == true { panic!("Began a new modification with one in progress!")}

        self.history.undo_stack.push(HistoryEntry {
            description: description.to_owned(),
            layer_idx: self.layer_idx,
            contour_idx: self.contour_idx,
            point_idx: self.point_idx,
            selected: Some(self.selected.clone()),
            layer: self.glyph.as_ref().unwrap().layers[self.layer_idx.unwrap()].clone(),
            kind: HistoryType::LayerModified
        });

        self.modifying = true;
    }

    /// Calls the closure with a mutable reference to the active layer as it's only argument.
    /// This is the primary function you should use in tools and contour operations to make changes to the
    /// glyph's state. This function will panic if you have not called begin_layer_modification!
    pub fn with_active_layer_mut<F, R>(&mut self, mut closure: F) -> R
        where F: FnMut(&mut Layer<MFEKPointData>) -> R {
        if self.modifying == false { panic!("A modification is not in progress!")}
        let glyph = self.glyph.as_mut().unwrap();
        let ret =closure(&mut glyph.layers[self.layer_idx.unwrap()]);

        self.rebuild_previews();
        ret
    }

    /// This ends an ongoing modification and calls the proper events.
    pub fn end_layer_modification(&mut self) {
        if self.modifying == false { return };

        // TODO: Events here.
        self.modifying = false;
        self.rebuild_previews();
    }

    pub fn is_modifying(&self) -> bool {
        self.modifying
    }

    /// Calls the supplied closure with an immutable reference to the active layer.
    pub fn with_active_layer<F, R>(&self, mut closure: F) -> R
        where F: FnMut(&Layer<MFEKPointData>) -> R {
        closure(&self.glyph.as_ref().unwrap().layers[self.layer_idx.unwrap()])
    }

    /// Calls the supplied closure with a copy of the glif.
    pub fn with_glyph<F, R>(&self, mut closure: F) -> R 
        where F: FnMut(&MFEKGlif<MFEKPointData>) -> R {
        closure(&self.glyph.as_ref().unwrap())
    }

    /// This function should not be called in tools or contour operations. TODO: Remove.
    pub fn with_glyph_mut<F, R>(&mut self, mut closure: F) -> R 
        where F: FnMut(&mut MFEKGlif<MFEKPointData>) -> R 
    {
        closure(&mut self.glyph.as_mut().unwrap())
    }

    /// Set the active tool by enum. When adding your own tools make sure to add them to ToolEnum.
    pub fn set_tool(&mut self, tool: ToolEnum ) {
        if self.active_tool_enum == tool { return };

        self.end_layer_modification();
        self.active_tool_enum = tool;
        self.active_tool = tool_enum_to_tool(tool);
    }

    /// Get the active tool by enum.
    pub fn get_tool(&self) -> ToolEnum {
        self.active_tool_enum
    }

    /// Get a mutable copy of the current tool as a boxed dyn Tool. This is used in event handling.
    pub fn get_tool_mut(&mut self) -> &mut Box<dyn Tool>
    {
        &mut self.active_tool
    }

    /// This function merges contour gracefully. This should be used over merging them yourself as it will automatically
    /// contour operations. This can only be called during a modification
    pub fn merge_contours(&mut self, start: usize, end: usize)
    {
        // TODO: fix contour operations

        // we're closing an open path
        if start == end {
            self.with_active_layer_mut(|layer| {
                let contour = get_contour_mut!(layer, start);
                let last_point = contour.pop().unwrap();

                contour.first_mut().unwrap().b = last_point.b;
                contour.first_mut().unwrap().ptype = glifparser::PointType::Curve;
            });
            self.point_idx = Some(0);
        } else {
            // we're merging two open paths
            let (cidx, pidx) = self.with_active_layer_mut(|layer| {
                let mut startc = get_contour_mut!(layer, start).clone();
                let endc = get_contour_mut!(layer, end);

                endc.last_mut().unwrap().b = startc[0].a;

                let p_idx = endc.len() - 1;
                startc.remove(0);
                for point in startc {
                    endc.push(point);
                }

                get_outline_mut!(layer).remove(start);
                //TODO: we need some kind of handling for when contours get removed
                if start < end {
                    (end, p_idx)
                } else {
                    (end, p_idx)
                }
            });

            self.contour_idx = Some(cidx);
            self.point_idx = Some(pidx);
        }
    }
}

thread_local!(pub static CONSOLE: RefCell<RendererConsole> = RefCell::new(RendererConsole::default()));
