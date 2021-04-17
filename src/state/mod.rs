//! Global thread local state.

use glifparser::{Contour, Glif, MFEKGlif, Outline, Point, PointType, WhichHandle, glif::{HistoryEntry, HistoryType, Layer}};
pub use crate::state::Follow;

pub use crate::renderer::console::Console as RendererConsole;
use crate::{events::{EditorEvent, Tool, ToolEnum, pan::Pan, tool_enum_to_tool}, renderer::Guideline};
use crate::get_outline;

use crate::renderer::constants;
use constants::{POINT_RADIUS, POINT_STROKE_THICKNESS};

pub use skulpin::skia_safe::Contains as _;
pub use skulpin::skia_safe::{Canvas, Matrix, Path as SkPath, Point as SkPoint, Rect as SkRect};
pub use crate::renderer::points::calc::*;


use std::{cell::RefCell, collections::{HashMap, HashSet}};
use std::path::PathBuf;

use crate::get_contour_mut;
use crate::get_outline_mut;
use crate::get_contour_len;


mod tool_data;
pub use self::tool_data::*;

mod toggles;
pub use self::toggles::*;

//TODO: Move to tool utility file
#[derive(PartialEq, Clone, Copy)]
pub enum SelectPointInfo {
    Start,
    End
}

pub struct EditorMods {
    pub shift: bool,
    pub ctrl: bool,
}

pub struct Glyph<P: glifparser::PointData> {
    pub glif: MFEKGlif<P>,
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
    
    pub modifiers: EditorMods,
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

    // this is your main selection or 'edit pointer'
    pub contour_idx: Option<usize>,   // index into Outline
    pub point_idx: Option<usize>, 
    pub layer_idx: Option<usize>, // active layer

    // this is drag selection
    pub selected: HashSet<(usize, usize)>,

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

            modifiers: EditorMods { shift: false, ctrl: false },
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
            selected: HashSet::new(),

            modifying: false,
        }
    }

    pub fn set_glyph(&mut self, glyph: Glyph<PointData>)
    {
        self.glyph = Some(glyph);
        self.layer_idx = Some(0);
    }

    pub fn get_layer_count(&self) -> usize {
        return self.glyph.as_ref().unwrap().glif.layers.len();
    }

    pub fn is_point_selected(&self, contour_idx: usize, point_idx: usize) -> bool
    {
        if let Some(editor_pidx) = self.point_idx {
            let editor_cidx = self.contour_idx.unwrap();

            if contour_idx == editor_cidx && point_idx == editor_pidx { return true };
        }

        if self.selected.contains(&(contour_idx, point_idx)) { return true };

        return false;
    }

    // TODO: split following 3 functions off into some tool utility file
    pub fn delete_selection(&mut self) {
        self.begin_layer_modification("Delete selection.");
        
        let layer = &self.glyph.as_ref().unwrap().glif.layers[self.layer_idx.unwrap()];
        let mut new_outline: Vec<Vec<Point<PointData>>> = Vec::new();
        for (contour_idx, contour) in layer.outline.as_ref().unwrap().iter().enumerate() {
            let mut results = Vec::new();
            let mut cur_contour = Vec::new();
            let mut deleted = false;
            for (point_idx, point) in contour.iter().enumerate() {
                let to_delete = self.is_point_selected(contour_idx, point_idx);

                if to_delete {
                    results.push(cur_contour);
                    cur_contour = Vec::new();
                    deleted = true;
                } else  {
                    cur_contour.push(point.clone());
                }
            }
            results.push(cur_contour);

            if results.len() > 1 && contour.first().unwrap().ptype != PointType::Move {
                let mut move_to_front = results.pop().unwrap().clone();
                move_to_front.append(&mut results[0]);
                results[0] = move_to_front;
            }

            for mut result in results {
                if result.len() != 0 {
                    if deleted {
                        result.first_mut().unwrap().ptype = PointType::Move;
                    }
                    new_outline.push(result); 
                }
            }
        }

        self.glyph.as_mut().unwrap().glif.layers[self.layer_idx.unwrap()].outline = Some(new_outline);

        self.end_layer_modification();

        self.contour_idx = None;
        self.point_idx = None;
        self.selected = HashSet::new();
    }
    /// Checks if the active point is the active contour's start or end. Does not modify.
    pub fn get_contour_start_or_end(&self, contour_idx: usize, point_idx: usize) -> Option<SelectPointInfo>
    {
        let contour_len = self.with_active_layer(|layer| {get_contour_len!(layer, contour_idx)} ) - 1;
        match point_idx {
            0 => Some(SelectPointInfo::Start),
            contour_len => Some(SelectPointInfo::End),
            _ => None
        }
    }

    /// Utility function to quickly check which point or mouse is hovering. Optional mask parameter specifies a point to ignore.
    pub fn clicked_point_or_handle(&self, mask: Option<(usize, usize)>) -> Option<(usize, usize, WhichHandle)> {
        let factor = self.factor;
        let _contour_idx = 0;
        let _point_idx = 0;

        // How we do this is quite naïve. For each click, we just iterate all points and check the
        // point and both handles. It's just a bunch of floating point comparisons in a compiled
        // language, so I'm not too concerned about it, and even in the TT2020 case doesn't seem to
        // slow anything down.
        self.with_active_layer(|layer| {
            for (contour_idx, contour) in get_outline!(layer).iter().enumerate() {
                for (point_idx, point) in contour.iter().enumerate() {
                    if let Some(mask) = mask { if contour_idx == mask.0 && point_idx == mask.1 { continue }};

                    let size = ((POINT_RADIUS * 2.) + (POINT_STROKE_THICKNESS * 2.)) * (1. / factor);
                    // Topleft corner of point
                    let point_tl = SkPoint::new(
                        calc_x(point.x as f32) - (size / 2.),
                        calc_y(point.y as f32) - (size / 2.),
                    );
                    let point_rect = SkRect::from_point_and_size(point_tl, (size, size));
                    // Topleft corner of handle a
                    let a = point.handle_or_colocated(WhichHandle::A, |f| f, |f| f);
                    let a_tl = SkPoint::new(calc_x(a.0) - (size / 2.), calc_y(a.1) - (size / 2.));
                    let a_rect = SkRect::from_point_and_size(a_tl, (size, size));
                    // Topleft corner of handle b
                    let b = point.handle_or_colocated(WhichHandle::B, |f| f, |f| f);
                    let b_tl = SkPoint::new(calc_x(b.0) - (size / 2.), calc_y(b.1) - (size / 2.));
                    let b_rect = SkRect::from_point_and_size(b_tl, (size, size));
        
                    // winit::PhysicalPosition as an SkPoint
                    let sk_mpos = SkPoint::new(self.mousepos.0 as f32, self.mousepos.1 as f32);
        
                    if point_rect.contains(sk_mpos) {
                        return Some((contour_idx, point_idx, WhichHandle::Neither));
                    } else if a_rect.contains(sk_mpos) {
                        return Some((contour_idx, point_idx, WhichHandle::A));
                    } else if b_rect.contains(sk_mpos) {
                        return Some((contour_idx, point_idx, WhichHandle::B));
                    }
                }
            }
            None
        })
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
    
    pub fn new_layer(&mut self) {
        let new_layer = Layer{
            outline: Some(Outline::new()),
            contour_ops: HashMap::new(),

        };

        self.history.push(HistoryEntry {
            description: "Added layer.".to_owned(),
            layer_idx: self.layer_idx,
            contour_idx: self.contour_idx,
            point_idx: self.point_idx,
            layer: new_layer.clone(), // dummy
            kind: HistoryType::LayerAdded
        });

        self.glyph.as_mut().unwrap().glif.layers.push(new_layer);
        
        self.end_layer_modification();

        self.layer_idx = Some(self.glyph.as_mut().unwrap().glif.layers.len() - 1);
        self.contour_idx = None;
        self.point_idx = None;
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
        where F: FnMut(&MFEKGlif<PointData>) -> R {
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
            match undo_entry.kind {
                HistoryType::LayerModified => {
                    self.glyph.as_mut().unwrap().glif.layers[undo_entry.layer_idx.unwrap()] = undo_entry.layer;
                    self.layer_idx = undo_entry.layer_idx;
                    self.contour_idx = undo_entry.contour_idx;
                    self.point_idx = undo_entry.point_idx;
                }
                HistoryType::LayerAdded => {
                    self.glyph.as_mut().unwrap().glif.layers.pop();
                    self.layer_idx = undo_entry.layer_idx;
                    self.contour_idx = undo_entry.contour_idx;
                    self.point_idx = undo_entry.point_idx;
                }
                HistoryType::LayerDeleted => {}
            }

        }
    }

    pub fn redo() {
        // TODO:
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointData;

thread_local!(pub static CONSOLE: RefCell<RendererConsole> = RefCell::new(RendererConsole::default()));
