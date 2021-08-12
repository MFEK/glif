use MFEKmath::{Bezier, Vector};
use flo_curves::bezier::solve_curve_for_t;
use glifparser::{Guideline, GuidelinePoint, IntegerOrFloat};

use crate::editor::Editor;
use crate::user_interface::Interface;
use crate::renderer::UIPointType;
use crate::renderer::points::draw_square_point;
use crate::skia_safe::Paint;
use super::prelude::*;
use crate::renderer::guidelines::draw_guideline;

mod dialog;
#[derive(Clone, PartialEq)]
pub enum GuidelineMode {
    Select,
    Add,
    Move,
}

#[derive(Clone)]
pub struct Guidelines {
    mode: GuidelineMode,
    selected_idx: Option<usize>,
    move_position: Option<(f32, f32)>
}

impl Tool for Guidelines {
    fn handle_event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent { event_type, meta } => {
                match event_type {
                    MouseEventType::Moved => { self.mouse_moved(v, i, meta) }
                    MouseEventType::Pressed => { self.mouse_pressed(v, i, meta) }
                    MouseEventType::Released => { self.mouse_released(v, i, meta) }
                    _ => {}
                }
            },
            EditorEvent::Draw { skia_canvas } => {
                self.draw_preview(i, skia_canvas);
                self.draw_selected_guideline(i, v, skia_canvas);
            },
            EditorEvent::Ui { ui}=> { self.tool_dialog(v, i, ui) },
            _ => {}
        }
    }
}

impl Guidelines {
    pub fn new() -> Self {
        Self {
            mode: GuidelineMode::Select,
            selected_idx: None,
            move_position: None,
        }
    }

    fn mouse_moved(&mut self, v: &mut Editor, i: &mut Interface, meta: MouseInfo) {
        if self.mode == GuidelineMode::Move {
            let mp = self.move_position.unwrap();
            let delta = (mp.0 - meta.position.0, mp.1 - meta.position.1);
            let selected = self.selected_idx.unwrap();

            v.with_glyph_mut(|glyph| {
                glyph.guidelines[selected].at.x -= delta.0;
                glyph.guidelines[selected].at.y += delta.1;
            });

            self.move_position = Some(meta.position);
        }
    }

    fn mouse_pressed(&mut self, v: &mut Editor, i: &mut Interface, meta: MouseInfo) {     
        if self.mode == GuidelineMode::Select {
            let vp = &i.viewport;
            v.with_glyph(|glyph| {
                for (idx, guide) in glyph.guidelines.iter().enumerate() {
                    let angle = guide.angle;
                    let at2 = Vector::from_components(
                        (guide.at.x+((1000.*vp.winsize.0 as f32)*f32::from(angle).cos())) as f64,
                        (guide.at.y+((1000.*vp.winsize.1 as f32)*f32::from(angle).sin())) as f64
                    );
                    let at3 = Vector::from_components(
                        (guide.at.x+((-(1000.*vp.winsize.0 as f32))*f32::from(angle).cos())) as f64,
                        (guide.at.y+((-(1000.*vp.winsize.1 as f32))*f32::from(angle).sin())) as f64
                    );

                    let position = i.mouse_info.position.clone();
                    let mouse_vec =  Vector::from_components(calc_x(position.0) as f64, calc_y(position.1 as f32) as f64);
                    let line_bez = Bezier::from_points(at2, at2, at3, at3);

                    if let Some(point_info) = solve_curve_for_t(&line_bez, &mouse_vec, 100000. / i.viewport.factor as f64) {
                        self.selected_idx = Some(idx);
                        self.move_position = Some(position);
                        self.mode = GuidelineMode::Move;
                    }
                }
            });
        }
    }

    fn mouse_released(&mut self, v: &mut Editor, i: &mut Interface, meta: MouseInfo) {
        if self.mode == GuidelineMode::Move {
            self.mode = GuidelineMode::Select;
            self.move_position = None;
        }

        if self.mode == GuidelineMode::Add {
            let abs_pos = i.mouse_info.absolute_position;
            let top = 0.;
            let left = 0.;
            let bottom = i.viewport.winsize.1 as f32;
            let right = i.viewport.winsize.0 as f32;
    
            let min_distance_vertical = f32::min(f32::abs(abs_pos.1 - bottom), f32::abs(abs_pos.1 - top));
            let min_distance_horizontal = f32::min(f32::abs(abs_pos.0 - left), f32::abs(abs_pos.0 - right));
        
            if min_distance_vertical < min_distance_horizontal {
                
                let prev_point = GuidelinePoint {
                    x: i.mouse_info.position.0,
                    y: calc_y(i.mouse_info.position.1),
                };
    
                let prev_guide = Guideline {
                    at: prev_point,
                    angle: IntegerOrFloat::Float(90.),
                    name: None,
                    color: None,
                    identifier: None,
                };
    
                v.with_glyph_mut(|glif| {
                    glif.guidelines.push(prev_guide.clone());
                });
            } else {
            
                let prev_point = GuidelinePoint {
                    x: i.mouse_info.position.0,
                    y: calc_y(i.mouse_info.position.1),
                };
    
                let prev_guide = Guideline {
                    at: prev_point,
                    angle: IntegerOrFloat::Float(0.),
                    name: None,
                    color: None,
                    identifier: None,
                };
    
                v.with_glyph_mut(|glif| {
                    glif.guidelines.push(prev_guide.clone());
                });
            }

            self.mode = GuidelineMode::Select;
        }
    }

    fn draw_preview(&mut self, i: &mut Interface, canvas: &mut Canvas) {
        if self.mode == GuidelineMode::Add {
            let abs_pos = i.mouse_info.absolute_position;
            let top = 0.;
            let left = 0.;
            let bottom = i.viewport.winsize.1 as f32;
            let right = i.viewport.winsize.0 as f32;
    
            let min_distance_vertical = f32::min(f32::abs(abs_pos.1 - bottom), f32::abs(abs_pos.1 - top));
            let min_distance_horizontal = f32::min(f32::abs(abs_pos.0 - left), f32::abs(abs_pos.0 - right));
        
            if min_distance_vertical < min_distance_horizontal {
                
                let prev_point = GuidelinePoint {
                    x: i.mouse_info.position.0,
                    y: calc_y(i.mouse_info.position.1),
                };
    
                let prev_guide = Guideline {
                    at: prev_point,
                    angle: IntegerOrFloat::Float(90.),
                    name: None,
                    color: None,
                    identifier: None,
                };
    
                draw_guideline(&i.viewport, canvas, &prev_guide, None)
            } else {
            
                let prev_point = GuidelinePoint {
                    x: i.mouse_info.position.0,
                    y: calc_y(i.mouse_info.position.1),
                };
    
                let prev_guide = Guideline {
                    at: prev_point,
                    angle: IntegerOrFloat::Float(0.),
                    name: None,
                    color: None,
                    identifier: None,
                };
    
                draw_guideline(&i.viewport, canvas, &prev_guide, None)
            }
        }
    }

    fn draw_selected_guideline(&mut self, i: &mut Interface, v: &mut Editor, canvas: &mut Canvas) {
        if let Some(selected) = self.selected_idx {
            v.with_glyph(|glif| {
                draw_guideline(&i.viewport, canvas, &glif.guidelines[selected], Some(SELECTED_FILL))
            })
        }
    }
}
