use crate::args::Args;
use crate::ipc;
use crate::tool_behaviors::ToolBehavior;
use crate::tools::{pan::Pan, Tool, ToolEnum};

use glifparser::MFEKPointData;
use glifparser::{
    glif::{HistoryEntry, Layer},
    Guideline, IntegerOrFloat, MFEKGlif,
};

pub use skia_safe::Contains as _;
pub use skia_safe::{Canvas, Matrix, Path as SkPath, Point as SkPoint, Rect as SkRect};

use std::collections::HashSet;
use std::path;
use std::sync::mpsc::{Receiver, Sender};

use self::{history::History, selection::EditorClipboard};

pub mod contour_handlers;
pub mod debug;
pub mod events;
pub mod filesystem_watch;
pub mod headless;
pub mod history;
pub mod images;
pub mod io;
pub mod layers;
pub mod operations;
pub mod selection;
pub mod tools;
pub mod util;

#[macro_use]
pub mod macros;

/// This is the main object that holds the state of the editor. It is responsible for mutating the glyph.
/// The only state that should change not through the editor is the generation of previews for the purposes of drawing.
#[derive(Debug)]
pub struct Editor {
    args: Args,
    glyph: Option<MFEKGlif<MFEKPointData>>,
    modifying: bool, // a flag that is set when the active layer is currently being modified

    dirty: bool, // Internal flag the editor uses to check for empty modifications.
    // end_layer_modification is called we simply discard the last history entry.
    history: History<MFEKPointData>, // holds a history of previous states the glyph has been in
    active_tool: Box<dyn Tool>,
    active_tool_enum: ToolEnum,
    clipboard: EditorClipboard,

    layer_idx: Option<usize>, // active layer
    pub contour_idx: Option<usize>, // index into Outline
    pub point_idx: Option<usize>,

    tool_behaviors: Vec<Box<dyn ToolBehavior>>,
    behavior_finished: bool,

    pub(crate) filesystem_watch_tx: Sender<path::PathBuf>,
    pub(crate) filesystem_watch_rx: Receiver<path::PathBuf>,

    preview_dirty: bool,
    pub preview: Option<MFEKGlif<MFEKPointData>>,

    pub italic_angle: f32,
    pub selected: HashSet<(usize, usize)>,

    pub images: images::EditorImages,
    // These are UFO-global guidelines which won't be picked up by glifparser.
    pub guidelines: Vec<Guideline<MFEKPointData>>,

    pub quit_requested: bool, // allows for quits from outside event loop, e.g. from command closures

    pub ipc_info: Option<mfek_ipc::IPCInfo>,
}

impl Editor {
    pub fn new(args: Args) -> Editor {
        let (fstx, fsrx) = std::sync::mpsc::channel();
        let mut self_o = Editor {
            args,
            glyph: None,
            modifying: false,
            dirty: false,
            history: History::default(),

            active_tool: Box::new(Pan::new()),
            active_tool_enum: ToolEnum::Pan,

            clipboard: EditorClipboard::default(),
            preview: None,

            layer_idx: None,
            contour_idx: None,
            point_idx: None,
            italic_angle: 0.,
            selected: HashSet::new(),

            images: images::EditorImages::new(),
            guidelines: vec![],
            quit_requested: false,
            ipc_info: None,
            preview_dirty: true,

            tool_behaviors: vec![],
            behavior_finished: true,

            filesystem_watch_tx: fstx,
            filesystem_watch_rx: fsrx,
        };
        self_o.headless();
        self_o
    }

    /// This function MUST be called before calling with_active_<layer/glif>_mut or it will panic.
    /// Pushes a clone of the current layer onto the history stack and puts the editor in a modifying state.
    /// When the fold argument is set to true the editor won't create new HistoryEntrys if the entry 
    /// below has the same description.
    pub fn begin_modification(&mut self, description: &str, fold: bool) {
        log::trace!("Modification begun: {}", description);
        if self.modifying {
            panic!("Began a new modification with one in progress!")
        }

        self.modifying = true;

        if let Some(last_entry) = self.history.undo_stack.last() {
            if fold && description.to_owned() == last_entry.description {
                return;
            }

        }

        self.history.add_undo_entry(HistoryEntry {
            description: description.to_owned(),
            layer_idx: self.layer_idx,
            contour_idx: self.contour_idx,
            point_idx: self.point_idx,
            guidelines: self.guidelines.clone(),
            selected: Some(self.selected.clone()),
            glyph: self.glyph.as_ref().unwrap().clone(),
        });

    }

    /// When calling this family of functions the editor will become inaccessible because of the borrow on one of it's members.
    /// It's best practice to call this function from within a block where the implicit drop will free the editor:
    /// {
    ///     let layer = get_active_layer_mut
    ///     //modify the layer here
    ///     //implicit drop at the end of scope releases borrow on editor
    /// }
    ///
    /// You can also use this function in one-liners like: self.get_active_layer_mut().some_field = blah
    /// The reference is implicitly dropped at the end of the statement.
    pub fn get_active_layer_mut(&mut self) -> &mut Layer<MFEKPointData> {
        if !self.modifying {
            panic!("A modification is not in progress!")
        }

        self.dirty = true;
        self.mark_preview_dirty();

        return &mut self.glyph.as_mut().unwrap().layers[self.layer_idx.unwrap()];
    }

    pub fn get_active_layer_ref(&self) -> &Layer<MFEKPointData> {
        return &self.glyph.as_ref().unwrap().layers[self.layer_idx.unwrap()];
    }

    /// This ends an ongoing modification and calls the proper events.
    pub fn end_modification(&mut self) {
        log::trace!("Ending modification…");
        if !self.modifying {
            log::error!("Tried to end a modification when not modifying!");
            return;
        }

        if let Some(history) = self.history.undo_stack.last() {
            log::trace!("Modification ended: {}", &history.description);
        }

        if !self.dirty {
            log::debug!("Ended a modification when editor did not think it was dirty");
        }

        // TODO: Events here.
        self.modifying = false;
        self.mark_preview_dirty();
    }

    pub fn is_modifying(&self) -> bool {
        self.modifying
    }

    pub fn add_width_guidelines(&mut self) {
        self.guidelines = self
            .guidelines
            .iter()
            .filter(|gl| {
                !gl.data.as_guideline().format
                    || (gl.data.as_guideline().format
                        && ["ascender", "descender"]
                            .iter()
                            .any(|gln| gl.name.as_ref().map(|n| n == gln).unwrap_or(false)))
            })
            .map(|gl| gl.clone())
            .collect();
        for (i, (x, y, name, angle)) in [
            (0., 0., "lbearing", IntegerOrFloat::Integer(90)),
            (
                self.glyph
                    .as_ref()
                    .map(|g| g.width.unwrap_or(0))
                    .unwrap_or(0) as f32,
                0.,
                "rbearing",
                IntegerOrFloat::Integer(90),
            ),
            (
                0.,
                0.,
                "italic_lbearing",
                IntegerOrFloat::Float(self.italic_angle),
            ),
            (
                self.glyph
                    .as_ref()
                    .map(|g| g.width.unwrap_or(0))
                    .unwrap_or(0) as f32,
                0.,
                "italic_rbearing",
                IntegerOrFloat::Float(self.italic_angle),
            ),
        ]
        .into_iter()
        .enumerate()
        {
            // Skip showing italic bearings if they'd be equal to normal bearings
            if self.italic_angle % 90. == 0. && i >= 2 {
                continue;
            }
            let (fixed, format, right) = (i % 2 == 0, true, i % 2 == 1);
            self.guidelines.insert(
                0,
                Guideline::from_x_y_angle(x, y, angle)
                    .name(name)
                    .data(MFEKPointData::new_guideline_data(fixed, format, right)),
            );
        }
    }

    pub fn set_glyph(&mut self, glyph: MFEKGlif<MFEKPointData>) {
        self.glyph = Some(glyph);
    }

    pub fn initialize(&mut self) {
        ipc::fetch_italic(self);
        self.guidelines.clear();
        self.add_width_guidelines();
        self.layer_idx = Some(0);
        self.mark_preview_dirty();
        self.recache_images();
        log::debug!("Images: {:?}", &self.images);
        ipc::fetch_metrics(self);
    }
}

macro_rules! yield_glyph_or_panic {
    ($self:ident) => {{
        if !$self.modifying {
            panic!("A modification is not in progress!")
        }

        $self.dirty = true;
        $self.mark_preview_dirty();
        $self.glyph.as_mut().unwrap()
    }};
}
// with_active_layer and friends
impl Editor {
    // Do not use this function unless you're very sure it's right.
    pub fn get_active_layer_mut_no_history(&mut self) -> &mut Layer<MFEKPointData> {
        if !self.modifying {
            panic!("A modification is not in progress!")
        }

        self.dirty = true;
        self.mark_preview_dirty();

        return &mut self.glyph.as_mut().unwrap().layers[self.layer_idx.unwrap()];
    }

    pub fn with_active_layer_mut_no_history<F, R>(&mut self, mut closure: F) -> R
    where
        F: FnMut(&mut Layer<MFEKPointData>) -> R,
    {
        static mut WARNED_HISTORY: bool = false;
        unsafe {
            if !WARNED_HISTORY {
                log::debug!(
                    "Used dangerous function: editor.with_active_layer_mut_no_history(|layer|…)"
                );
            }
            WARNED_HISTORY = true;
        }
        let glyph = self.glyph.as_mut().unwrap();
        closure(&mut glyph.layers[self.layer_idx.unwrap()])
    }
}

// with_glyph and friends
impl Editor {
    /// Calls the supplied closure with a copy of the glif.
    pub fn with_glyph<F, R>(&self, mut closure: F) -> R
    where
        F: FnMut(&MFEKGlif<MFEKPointData>) -> R,
    {
        closure(self.glyph.as_ref().unwrap())
    }

    /// This function should be used to modify anchors, guidelines, and other glyph-level data. Do not use this to modify the active layer.
    pub fn with_glyph_mut<F, R>(&mut self, mut closure: F) -> R
    where
        F: FnMut(&mut MFEKGlif<MFEKPointData>) -> R,
    {
        closure(yield_glyph_or_panic!(self))
    }

    // As for layers, the purpose of this function is preventing expensive clone operations.
    pub fn with_glyph_mut_and_owned_data<F, R, T>(&mut self, mut closure: F, data: T) -> R
    where
        F: FnMut(&mut MFEKGlif<MFEKPointData>, T) -> R,
    {
        let ret = closure(yield_glyph_or_panic!(self), data);
        self.mark_preview_dirty();
        ret
    }

    // Do not use this function unless you're very sure it's right.
    pub fn with_glyph_mut_no_history<F, R>(&mut self, mut closure: F) -> R
    where
        F: FnMut(&mut MFEKGlif<MFEKPointData>) -> R,
    {
        log::trace!("Used dangerous function: editor.with_glyph_mut_no_history(|glyph|…)");
        closure(self.glyph.as_mut().unwrap())
    }
}

// TODO: Reenable console.
//thread_local!(pub static CONSOLE: RefCell<RendererConsole> = RefCell::new(RendererConsole::default()));
