mod dialog;

use std::rc::Rc;

use glifparser::glif::{ContourOperations, PAPContour, PatternCopies, PatternStretch, PatternSubdivide};

use super::prelude::*;
use crate::{editor::Editor, user_interface::InputPrompt};

#[derive(Clone, Debug)]
pub struct PAP {}

impl Tool for PAP {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent {
                event_type,
                mouse_info,
            } => match event_type {
                MouseEventType::Pressed => self.mouse_pressed(v, i, mouse_info),
                _ => {}
            },
            _ => {}
        }
    }

    fn ui(&mut self, v: &mut Editor, i: &mut Interface, ui: &mut Ui) {
        self.tool_dialog(v, i, ui)
    }
}

impl PAP {
    pub fn new() -> Self {
        Self {}
    }

    fn mouse_pressed(&mut self, v: &mut Editor, i: &mut Interface, mouse_info: MouseInfo) {
        if let Some((ci, pi, _wh)) = clicked_point_or_handle(v, i, mouse_info.raw_position, None) {
            let layer_op = v.with_active_layer(|layer| layer.outline[ci].operation.clone());
            if let Some(_op) = layer_op {
            } else {
                i.push_prompt(InputPrompt::Layer {
                    label: "Select a pattern.".to_string(),
                    func: Rc::new(move |editor, source_layer| {
                        editor.contour_idx = Some(ci);
                        editor.point_idx = Some(pi);

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
