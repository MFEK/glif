use crate::{get_point_mut, editor::tunni::{TunniLineInfo, get_tunni_line_from_info}};

use super::prelude::*;
use MFEKmath::Vector;
use glifparser::glif::mfek::contour::MFEKContourCommon;

#[derive(Clone, Debug)]
pub struct MoveTunniPoint {
    tunni_info: TunniLineInfo
}

impl ToolBehavior for MoveTunniPoint {
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

impl MoveTunniPoint {
    pub fn new(_: MouseInfo, tunni_info: TunniLineInfo) -> Self {
        MoveTunniPoint {
            tunni_info
        }
    }

    pub fn mouse_moved(&mut self, v: &mut Editor, _: &mut Interface, mouse_info: MouseInfo) {
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

        // the new tunni point
        let t = Vector::from_components(mouse_info.position.0 as f64, mouse_info.position.1 as f64);
        let a = Vector::from_point(&a_point);
        let b = Vector::from_point(&b_point);

        let c1 = Vector::from_handle(&a_point, a_point.a);
        let c2 =  Vector::from_handle(&b_point, b_point.b);

        let ha = (a + t) / 2.;
        let hb = (b + t) / 2.;

        let ha1 = ha + c2 - b;
        let hb1 = hb + c1 - a;

        let a_intersect = flo_curves::line::ray_intersects_ray(&(a, c1), &(ha1, ha));
        let b_intersect = flo_curves::line::ray_intersects_ray(&(b, c2), &(hb1, hb),);

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