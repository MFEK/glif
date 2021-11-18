mod dialog;

use std::rc::Rc;

use glifparser::glif::{
    ContourOperations, PAPContour, PatternCopies, PatternStretch, PatternSubdivide,
};

use super::prelude::*;
use crate::{editor::Editor, user_interface::InputPrompt};

#[derive(Clone, Debug)]
pub struct PAP {}

impl Tool for PAP {
    #[rustfmt::skip]
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        if let EditorEvent::MouseEvent { mouse_info, event_type } = event {
            match event_type {
                MouseEventType::Pressed => self.mouse_pressed(v, i, mouse_info),
                _ => (),
            }
        }
    }

    fn ui(&mut self, v: &mut Editor, i: &mut Interface, ui: &mut Ui) {
        let show_dialog = v.with_active_layer(|layer| match v.contour_idx {
            Some(ci) => match layer.outline[ci].operation {
                Some(ContourOperations::PatternAlongPath { .. }) => true,
                _ => false,
            },
            _ => false,
        });
        if show_dialog {
            self.tool_dialog(v, i, ui);
        }
    }
}

impl PAP {
    pub fn new() -> Self {
        Self {}
    }

    fn mouse_pressed(&mut self, v: &mut Editor, i: &mut Interface, mouse_info: MouseInfo) {
        if let Some((ci, pi, _wh)) = clicked_point_or_handle(v, i, mouse_info.raw_position, None) {
            v.contour_idx = Some(ci);
            v.point_idx = Some(pi);

            let layer_op = v.with_active_layer(|layer| layer.outline[ci].operation.clone());

            match layer_op {
                Some(ContourOperations::PatternAlongPath { .. }) => (),
                None | Some(_) => {
                    i.push_prompt(InputPrompt::Layer {
                        label: "Select a pattern.".to_string(),
                        func: Rc::new(move |editor, source_layer| {
                            editor.begin_modification("Added PAP contour.");
                            editor.with_active_layer_mut(|layer| {
                                layer.outline[ci].operation =
                                    Some(ContourOperations::PatternAlongPath {
                                        // TODO: Default() implementation for many of our structs.
                                        data: PAPContour {
                                            pattern: source_layer.outline.clone(),
                                            copies: PatternCopies::Repeated,
                                            subdivide: PatternSubdivide::Off,
                                            is_vertical: false,
                                            stretch: PatternStretch::On,
                                            spacing: 4.,
                                            simplify: false,
                                            normal_offset: 0.,
                                            tangent_offset: 0.,
                                            pattern_scale: (1., 1.),
                                            center_pattern: true,
                                            prevent_overdraw: 0.,
                                            two_pass_culling: false,
                                            reverse_path: false,
                                            reverse_culling: false,
                                        },
                                    })
                            });
                            editor.end_modification();
                        }),
                    });
                }
            }
        }
    }
}
