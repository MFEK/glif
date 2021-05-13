use std::rc::Rc;

use glifparser::glif::{ContourOperations, PAPContour, PatternCopies, PatternSubdivide};
use imgui::Ui;

use crate::{contour_operations::ContourOperation, editor::Editor, user_interface::{self, InputPrompt}};
use super::prelude::*;

#[derive(Clone)]
pub struct PAP {}

impl Tool for PAP {
    fn handle_event(&mut self, v: &mut Editor, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent { event_type, meta } => {
                match event_type {
                    MouseEventType::Moved => { self.mouse_moved(v, meta) }
                    MouseEventType::Pressed => { self.mouse_pressed(v, meta) }
                    MouseEventType::Released => { self.mouse_released(v, meta) }
                    _ => {}
                }
            }
            EditorEvent::Ui { ui}=> { self.tool_dialog(v, ui) }
            _ => {}
        }
    }
}

impl PAP {
    pub fn new() -> Self {
        Self {}
    }

    fn mouse_moved(&mut self, v: &mut Editor, meta: MouseInfo) {
    }

    fn mouse_pressed(&mut self, v: &mut Editor, meta: MouseInfo) {
        if let Some((ci, pi, wh)) = clicked_point_or_handle(v, meta.position, None) {
            let layer_op = v.with_active_layer(|layer| layer.outline[ci].operation.clone() );
            if let Some(op) = layer_op {
                
            }
            else {
                v.prompts.push(InputPrompt::Layer{
                    label: "Select a pattern.".to_string(),
                    func: Rc::new(move |editor, source_layer| {
                        editor.contour_idx = Some(ci);
                        editor.point_idx = Some(pi);

                        editor.begin_layer_modification("Added PAP contour.");
                        editor.with_active_layer_mut(|layer| {
                            layer.outline[ci].operation = Some(ContourOperations::PatternAlongPath {
                                data: PAPContour {
                                    pattern: source_layer.outline.clone(),
                                    copies: PatternCopies::Repeated,
                                    subdivide: PatternSubdivide::Off,
                                    is_vertical: false,
                                    stretch: false,
                                    spacing: 4.,
                                    simplify: false,
                                    normal_offset: 0.,
                                    tangent_offset: 0.,
                                    pattern_scale: (1., 1.),
                                    center_pattern: true,
                                }
                            })
                        });
                        editor.end_layer_modification();
                    }),
                });
            }
        }   
    }

    fn mouse_released(&mut self, _v: &mut Editor, _meta: MouseInfo) {
    }

    fn tool_dialog(&mut self, v: &mut Editor, ui: &Ui) {
        let (tx, ty, tw, th) = user_interface::get_tools_dialog_rect(v);

        imgui::Window::new(&imgui::ImString::new("Pattern Along Path"))
        .bg_alpha(1.) // See comment on fn redraw_skia
        .flags(
            #[rustfmt::skip]
              imgui::WindowFlags::NO_RESIZE
                | imgui::WindowFlags::NO_MOVE
                | imgui::WindowFlags::NO_COLLAPSE,
        )
        .position(
            [tx, ty],
            imgui::Condition::Always,
        )
        .size(
            [tw, th],
            imgui::Condition::Always,
        )
        .build(ui, || {
            
        });
    }
    fn generate_PAP_contour()
    {

    }
}
