use MFEKmath::Vector;

use crate::get_point_mut;

use super::prelude::*;
use glifparser::glif::contour::MFEKContourCommon;

#[derive(Clone, Debug)]
pub struct RotateSelection {
    pivot_point: (f32, f32),
    rotate_vector: (f32, f32),
    mouse_info: MouseInfo,
}

impl RotateSelection {
    pub fn new(pivot_point: (f32, f32), rotate_vector: (f32, f32), mouse_info: MouseInfo) -> Self {
        RotateSelection {
            pivot_point,
            mouse_info,
            rotate_vector,
        }
    }

    pub fn mouse_moved(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        if !v.is_modifying() {
            v.begin_modification("Rotate selection.", false);
        }

        let rot = self.rotate_vector;
        let pivot = self.pivot_point;
        let raw_pivot_vector = Vector::from_components(pivot.0 as f64, pivot.1 as f64);
        let pivot_vector = Vector::from_components(pivot.0 as f64, pivot.1 as f64);
        let mouse_vector =
            Vector::from_components(mouse_info.position.0 as f64, mouse_info.position.1 as f64);

        let normal_from_pivot = (pivot_vector - mouse_vector).normalize();

        let rot_vec = Vector::from_components(rot.0 as f64, rot.1 as f64);
        let rotation_angle = normal_from_pivot.angle(rot_vec);

        for (ci, pi) in &v.selected.clone() {
            {
                let layer = v.get_active_layer_mut();
                let point = get_point_mut!(layer, *ci, *pi).unwrap();
                let point_vec = Vector::from_components(point.x() as f64, point.y() as f64);
                let rotated_point = point_vec.rotate(raw_pivot_vector, rotation_angle);
                
                point.set_position_no_handles(rotated_point.x as f32, rotated_point.y as f32);

                if let Some(a_pos) = point.get_handle_position(WhichHandle::A) {
                    let a_vec = Vector::from_components(a_pos.0 as f64, a_pos.1 as f64);
                    let rotated_a = a_vec.rotate(raw_pivot_vector, rotation_angle);

                    point.set_handle_position(WhichHandle::A, rotated_a.x as f32, rotated_a.y as f32);
                }

                if let Some(b_pos) = point.get_handle_position(WhichHandle::B) {
                    let b_vec = Vector::from_components(b_pos.0 as f64, b_pos.1 as f64);
                    let rotated_b = b_vec.rotate(raw_pivot_vector, rotation_angle);

                    point.set_handle_position(WhichHandle::A, rotated_b.x as f32, rotated_b.y as f32);
                }
            }
        }

        self.rotate_vector = normal_from_pivot.into();
    }

    pub fn mouse_released(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        if mouse_info.button == self.mouse_info.button {
            v.end_modification();
            v.pop_behavior();
        }
    }
}

#[rustfmt::skip]
impl ToolBehavior for RotateSelection {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        if let EditorEvent::MouseEvent { mouse_info, event_type } = event {
            match event_type {
                MouseEventType::Released => self.mouse_released(v, i, mouse_info),
                MouseEventType::Moved => self.mouse_moved(v, i, mouse_info),
                _ => (),
            }
        }
    }
}
