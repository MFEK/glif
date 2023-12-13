use crate::{get_point_mut, editor::tunni::{TunniLineInfo, construct_tunni_line, get_tunni_line_from_info}};

use super::prelude::*;
use MFEKmath::Vector;
use glifparser::glif::{mfek::contour::MFEKContourCommon};
#[derive(Clone, Debug)]
pub struct MoveTunniLine {
    mouse_info: MouseInfo,
    tunni_info: TunniLineInfo,
    line: (Vector, Vector) // The starting line from first.a(c1) -> second.b(c2)
}

impl ToolBehavior for MoveTunniLine {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent { event_type, mouse_info } => {
                match event_type {
                    MouseEventType::Moved => self.mouse_moved(v, i, mouse_info),
                    MouseEventType::Released => v.pop_behavior(),
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

impl MoveTunniLine {
    pub fn new(mouse_info: MouseInfo, tunni_info: TunniLineInfo, line: (Vector, Vector)) -> Self {
        MoveTunniLine {
            mouse_info,
            tunni_info,
            line
        }
    }

    pub fn mouse_moved(&mut self, v: &mut Editor, i: &mut Interface, mouse_info: MouseInfo) {
        // we stop the drag when there's no longer a well formed tunni point/line
        if get_tunni_line_from_info(v, &self.tunni_info).is_none() {
            v.pop_behavior();
        }

        let a_point = 
            get_point!(v.get_active_layer_ref(), self.tunni_info.a.0, self.tunni_info.a.1)
                .unwrap()
                .cubic()
                .expect("A tunni line info should never index to a non-cubic point!")
                .clone();

        let b_point = 
            get_point!(v.get_active_layer_ref(), self.tunni_info.b.0, self.tunni_info.b.1)
                .unwrap()
                .cubic()
                .expect("A tunni line info should never index to a non-cubic point!")
                .clone();

        let a = Vector::from_point(&a_point);
        let b = Vector::from_point(&b_point);

        let c1 = Vector::from_handle(&a_point, a_point.a);
        let c2 =  Vector::from_handle(&b_point, b_point.b);

        let starting_mp: Vector = self.mouse_info.position.into();
        let current_mp: Vector = mouse_info.position.into();

        let delta = current_mp - starting_mp;
        let offset_c1 = self.line.0 + delta;
        let offset_c2 = self.line.1 + delta;

        let a_intersect = flo_curves::line::ray_intersects_ray(&(a, c1), &(offset_c2, offset_c1));
        let b_intersect = flo_curves::line::ray_intersects_ray(&(b, c2), &(offset_c1, offset_c2),);

        match (a_intersect, b_intersect) {
            (Some(c1), Some(c2)) => {
                v.begin_modification("Move tunni point.", true);

                get_point_mut!(v.get_active_layer_mut(), self.tunni_info.a.0, self.tunni_info.a.1)
                    .unwrap()
                    .set_handle(WhichHandle::A, Handle::At(c1.x as f32, c1.y as f32));

                get_point_mut!(v.get_active_layer_mut(), self.tunni_info.b.0, self.tunni_info.b.1)
                    .unwrap()
                    .set_handle(WhichHandle::B, Handle::At(c2.x as f32, c2.y as f32));

                v.end_modification();
            },
            _ => {}
        }
        
        
    }
}