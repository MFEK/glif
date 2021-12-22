use glifrenderer::guidelines::draw_guideline;

use super::prelude::*;
use crate::editor::Editor;
use crate::tool_behaviors::{
    move_glyph::MoveGlyph, move_guideline::MoveGuideline, zoom_scroll::ZoomScroll,
};
use crate::user_interface::Interface;
use crate::util::MFEKGlifPointData;

use flo::Coordinate;
use flo_curves as flo;
use glifparser::Guideline;
use log;

mod dialog;

#[derive(Clone, Debug, Default)]
pub struct Guidelines {
    selected_idx: Option<usize>,
}

#[derive(Debug, Clone, Default)]
struct SplitGuidelines {
    guidelines: Vec<(Guideline<MFEKGlifPointData>, bool)>,
    local_guidelines: Vec<Guideline<MFEKGlifPointData>>,
    global_guidelines: Vec<Guideline<MFEKGlifPointData>>,
}

use itertools::{Either, Itertools as _};

impl SplitGuidelines {
    fn new(v: &Editor) -> Self {
        let mut ret = Self::default();
        ret.guidelines = v.with_glyph(|glyph| {
            glyph
                .guidelines
                .iter()
                .map(|g| g.clone())
                .collect::<Vec<_>>()
                .iter()
                .map(|g| (g.to_owned(), false))
                .chain(v.guidelines.iter().map(|g| (g.to_owned(), true)))
                .collect::<Vec<_>>()
        });
        let (local_guidelines, global_guidelines) =
            ret.guidelines.iter().partition_map(|(g, global)| {
                if !global {
                    Either::Left(g.clone())
                } else {
                    Either::Right(g.clone())
                }
            });
        ret.local_guidelines = local_guidelines;
        ret.global_guidelines = global_guidelines;

        ret
    }

    // used as:
    //
    //     let (mut guidelines, guidelines_len, local_guidelines_len, global_guidelines_len) = SplitGuidelines::new(v).as_tuple();
    //
    fn as_tuple(
        self,
    ) -> (
        Vec<(Guideline<MFEKGlifPointData>, bool)>,
        usize,
        usize,
        usize,
    ) {
        let l = self.guidelines.len();
        (
            self.guidelines,
            l,
            self.local_guidelines.len(),
            self.global_guidelines.len(),
        )
    }
}

impl Tool for Guidelines {
    #[rustfmt::skip]
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::IOEvent { event_type: IOEventType::FontinfoReloaded, .. } => {
                self.selected_idx = None;
            }
            EditorEvent::MouseEvent { mouse_info, event_type } => match event_type {
                MouseEventType::Pressed => self.mouse_pressed(v, i, mouse_info),
                _ => (),
            }
            EditorEvent::ScrollEvent { .. } => ZoomScroll::default().event(v, i, event),
            _ => {}
        }
    }

    fn draw(&self, v: &Editor, i: &Interface, canvas: &mut Canvas) {
        self.draw_selected_guideline(i, v, canvas);
    }

    fn ui(&mut self, v: &mut Editor, i: &mut Interface, ui: &mut Ui) {
        self.tool_dialog(v, i, ui)
    }
}

impl Guidelines {
    pub fn new() -> Self {
        Self::default()
    }

    // This function searches for a guideline to select based on mouse position.
    fn mouse_pressed(&mut self, v: &mut Editor, i: &mut Interface, mouse_info: MouseInfo) {
        // Otherwise, deselect so if we've a guideline selected it will no longer be.
        self.selected_idx = None;

        let split_guidelines = SplitGuidelines::new(v);
        let (guidelines, _guidelines_len, _local_guidelines_len, _global_guidelines_len) =
            split_guidelines.as_tuple();

        for (idx, (guide, _)) in guidelines.iter().enumerate() {
            let position = i.mouse_info.position;
            log::trace!(
                "Guidelines::mouse_pressed(…): Trying guideline index {} ({:?})",
                idx,
                &guide
            );
            // Vectorize guideline angle
            let angle = f64::from(-guide.angle);
            let angle_vec = kurbo::Vec2::from_angle(angle.to_radians());
            // Vectorize angle perpendicular to guideline angle
            let angle2 = f64::from(-guide.angle) + 90.;
            let angle2_vec = kurbo::Vec2::from_angle(angle2.to_radians());
            let vec = kurbo::Vec2::from((guide.at.x as f64, guide.at.y as f64));
            let vec2 = kurbo::Vec2::from((vec.x + angle_vec.x, vec.y + angle_vec.y));
            // Create line segments based on the position of the mouse and both above angles
            let position_angle = (
                position.0 + angle_vec.x as f32,
                position.1 + angle_vec.y as f32,
            );
            let position_angle2 = (
                position.0 + angle2_vec.x as f32,
                position.1 + angle2_vec.y as f32,
            );
            // Create three points: the point at the guideline's location, and the point at the
            // guideline's location offset by its angle. This is for creating a line segment which
            // flo_curves turns into a `ray`, its word for an infinite line in Euclidean maths.
            let p1 =
                flo::geo::Coord2::from((calc_x(vec.x as f32) as f64, calc_y(vec.y as f32) as f64));
            let p2 = flo::geo::Coord2::from((
                calc_x(vec2.x as f32) as f64,
                calc_y(vec2.y as f32) as f64,
            ));
            let pp = flo::geo::Coord2::from((position.0, position.1));
            // Find the mouse's distance from the guideline in question, along perpendicular axes
            let inter = flo::line::ray_intersects_ray(
                &(p1, p2),
                &(
                    flo::geo::Coord2::from(position),
                    flo::geo::Coord2::from(position_angle),
                ),
            );
            let inter2 = flo::line::ray_intersects_ray(
                &(p1, p2),
                &(
                    flo::geo::Coord2::from(position),
                    flo::geo::Coord2::from(position_angle2),
                ),
            );

            // For both distances considered…
            for rir in [inter, inter2] {
                if let Some(p) = rir {
                    // If either is within 5 screen units, select the point and get it ready for
                    // moving.
                    if p.distance_to(&pp) < 5. / i.viewport.factor as f64 {
                        let selected_idx = idx;
                        self.selected_idx = Some(selected_idx);
                        log::debug!(
                            "Guidelines::mouse_pressed(…): self.selected_index = {}",
                            idx
                        );
                        break;
                    }
                }
            }
        }

        // Finally, push the proper behavior if we found a point above.
        if let Some(selected_idx) = self.selected_idx {
            if guidelines[selected_idx].0.data.as_guideline().fixed {
                let mut behavior = Box::new(MoveGlyph::default());
                behavior.mouse_pressed(v, i, mouse_info);
                v.set_behavior(behavior);
            } else {
                let behavior = Box::new(MoveGuideline {
                    selected_idx,
                    global: guidelines[selected_idx].1,
                    mouse_info,
                });
                v.set_behavior(behavior);
            }
        }
    }

    fn draw_selected_guideline(&self, i: &Interface, v: &Editor, canvas: &mut Canvas) {
        let split_guidelines = SplitGuidelines::new(v);

        if let Some(selected) = self.selected_idx {
            draw_guideline(
                &i.viewport,
                canvas,
                &split_guidelines.guidelines[selected].0,
                Some(SELECTED_FILL),
            );
        }
    }
}
