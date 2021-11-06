use crate::{
    tool_behaviors::ToolBehavior,
    tools::{pan::Pan, Tool, ToolEnum},
};
use glifparser::{
    glif::{HistoryEntry, Layer, MFEKPointData},
    Guideline, MFEKGlif,
};

pub use skulpin::skia_safe::Contains as _;
pub use skulpin::skia_safe::{Canvas, Matrix, Path as SkPath, Point as SkPoint, Rect as SkRect};

use std::collections::HashSet;

use crate::editor::history::History;
use crate::get_contour_mut;

pub mod debug;
pub mod io;
pub mod headless;
pub mod history;
pub mod images;
pub mod layers;
pub mod operations;
pub mod selection;
pub mod tools;
pub mod util;

#[macro_use]
pub mod macros;

/// This is the main object that holds the state of the editor. It is responsible for mutating the glyph.
/// The only state that should change not through the editor is the generation of previews for the purposes of drawing.
pub struct Editor {
    glyph: Option<MFEKGlif<MFEKPointData>>,
    modifying: bool, // a flag that is set when the active layer is currently being modified

    dirty: bool, // Internal flag the editor uses to check for empty modifications.
    // end_layer_modification is called we simply discard the last history entry.
    history: History, // holds a history of previous states the glyph has been in
    active_tool: Box<dyn Tool>,
    active_tool_enum: ToolEnum,
    clipboard: Option<Layer<MFEKPointData>>,
    layer_idx: Option<usize>, // active layer
    preview_dirty: bool,

    tool_behaviors: Vec<Box<dyn ToolBehavior>>,
    behavior_finished: bool,

    pub preview: Option<MFEKGlif<MFEKPointData>>,
    pub contour_idx: Option<usize>, // index into Outline
    pub point_idx: Option<usize>,
    pub selected: HashSet<(usize, usize)>,

    pub images: images::EditorImages,
    // These are UFO-global guidelines which won't be picked up by glifparser.
    pub guidelines: Vec<Guideline>,

    pub quit_requested: bool, // allows for quits from outside event loop, e.g. from command closures

    pub ipc_info: Option<mfek_ipc::IPCInfo>,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            glyph: None,
            modifying: false,
            dirty: false,
            history: History::new(),

            active_tool: Box::new(Pan::new()),
            active_tool_enum: ToolEnum::Pan,

            clipboard: None,
            preview: None,

            layer_idx: None,
            contour_idx: None,
            point_idx: None,
            selected: HashSet::new(),

            images: images::EditorImages::new(),
            guidelines: vec![],
            quit_requested: false,
            ipc_info: None,
            preview_dirty: true,

            tool_behaviors: vec![],
            behavior_finished: true,
        }
    }

    /// This function MUST be called before calling with_active_<layer/glif>_mut or it will panic.
    /// Pushes a clone of the current layer onto the history stack and puts the editor in a modifying state.
    pub fn begin_modification(&mut self, description: &str) {
        if self.modifying == true {
            panic!("Began a new modification with one in progress!")
        }

        self.history.add_undo_entry(HistoryEntry {
            description: description.to_owned(),
            layer_idx: self.layer_idx,
            contour_idx: self.contour_idx,
            point_idx: self.point_idx,
            selected: Some(self.selected.clone()),
            glyph: self.glyph.as_ref().unwrap().clone(),
        });

        self.modifying = true;
    }

    /// This ends an ongoing modification and calls the proper events.
    pub fn end_modification(&mut self) {
        if self.modifying == false {
            return;
        };

        if !self.dirty {
            self.history.undo_stack.pop();
        }

        // TODO: Events here.
        self.modifying = false;
        self.mark_preview_dirty();
    }

    pub fn is_modifying(&self) -> bool {
        self.modifying
    }

    /// This function merges contour gracefully. This should be used over merging them yourself as it will automatically
    /// deal with contour operations. This can only be called during a modification
    pub fn merge_contours(&mut self, start: usize, end: usize) {
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

                layer.outline.remove(start);
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

    pub fn set_glyph(&mut self, glyph: MFEKGlif<MFEKPointData>) {
        self.glyph = Some(glyph);
        self.layer_idx = Some(0);
        self.mark_preview_dirty();
        self.recache_images();
        log::debug!("Images: {:?}", &self.images);
    }

    /// Calls the supplied closure with an immutable reference to the active layer.
    pub fn with_active_layer<F, R>(&self, mut closure: F) -> R
    where
        F: FnMut(&Layer<MFEKPointData>) -> R,
    {
        closure(&self.glyph.as_ref().unwrap().layers[self.layer_idx.unwrap()])
    }

    /// Calls the closure with a mutable reference to the active layer as it's only argument.
    /// This is the primary function you should use in tools and contour operations to make changes to the
    /// glyph's state. This function will panic if you have not called begin_layer_modification!
    pub fn with_active_layer_mut<F, R>(&mut self, mut closure: F) -> R
    where
        F: FnMut(&mut Layer<MFEKPointData>) -> R,
    {
        if self.modifying == false {
            panic!("A modification is not in progress!")
        }

        self.dirty = true;
        let glyph = self.glyph.as_mut().unwrap();
        let ret = closure(&mut glyph.layers[self.layer_idx.unwrap()]);

        self.mark_preview_dirty();
        ret
    }

    /// Calls the supplied closure with a copy of the glif.
    pub fn with_glyph<F, R>(&self, mut closure: F) -> R
    where
        F: FnMut(&MFEKGlif<MFEKPointData>) -> R,
    {
        closure(&self.glyph.as_ref().unwrap())
    }

    /// This function should be used to modify anchors, guidelines, and other glyph-level data. Do not use this to modify the active layer.
    pub fn with_glyph_mut<F, R>(&mut self, mut closure: F) -> R
    where
        F: FnMut(&mut MFEKGlif<MFEKPointData>) -> R,
    {
        if self.modifying == false {
            panic!("A modification is not in progress!")
        }

        self.dirty = true;
        self.mark_preview_dirty();
        let ret = closure(&mut self.glyph.as_mut().unwrap());
        ret
    }

    pub fn with_glyph_mut_no_history<F, R>(&mut self, mut closure: F) -> R
    where
        F: FnMut(&mut MFEKGlif<MFEKPointData>) -> R,
    {
        closure(&mut self.glyph.as_mut().unwrap())
    }
}

// TODO: Reenable console.
//thread_local!(pub static CONSOLE: RefCell<RendererConsole> = RefCell::new(RendererConsole::default()));
