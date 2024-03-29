use std::convert::TryInto;

use kurbo::Affine;
use MFEKmath::Vector;

use super::prelude::*;

#[derive(Clone, Debug)]
pub struct RotateImage {
    selected_idx: usize,
    pivot: (f32, f32),
    rotate_vector: (f32, f32),
    mouse_info: MouseInfo,
}

impl RotateImage {
    pub fn new(selected_idx: usize, pivot: (f32, f32), mouse_info: MouseInfo) -> Self {
        let pivot_vector = Vector::from_components(pivot.0 as f64, pivot.1 as f64);
        let mouse_vector =
            Vector::from_components(mouse_info.position.0 as f64, mouse_info.position.1 as f64);
        let rotate_vector = (pivot_vector - mouse_vector).normalize().into();

        RotateImage {
            pivot,
            rotate_vector,
            mouse_info,
            selected_idx,
        }
    }

    pub fn mouse_moved(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        if !v.is_modifying() {
            v.begin_modification("Rotate image.", false)
        }
        let pivot_vector = Vector::from_components(self.pivot.0 as f64, self.pivot.1 as f64);
        let mouse_vector =
            Vector::from_components(mouse_info.position.0 as f64, mouse_info.position.1 as f64);

        let normal_from_pivot = (pivot_vector - mouse_vector).normalize();

        let rot_vec =
            Vector::from_components(self.rotate_vector.0 as f64, self.rotate_vector.1 as f64);
        let rotation_angle = normal_from_pivot.angle(rot_vec);

        self.rotate_vector = normal_from_pivot.into();

        {
            let layer = v.get_active_layer_mut();
            let affine = layer.images[self.selected_idx].1;
            let raw_affine: Vec<f32> = affine.as_coeffs().iter().map(|x| *x as f32).collect();

            let sk_affine = Matrix::from_affine(&raw_affine.try_into().unwrap());
            let rotate_mat =
                Matrix::rotate_deg_pivot(-(rotation_angle as f32).to_degrees(), self.pivot);

            let sk_affine = rotate_mat * sk_affine;

            let translated_raw_affine = sk_affine.to_affine();

            if let Some(tra) = translated_raw_affine {
                let tra: Vec<f64> = tra.iter().map(|x| *x as f64).collect();
                layer.images[self.selected_idx].1 = Affine::new(tra.try_into().unwrap());
            }
        }

        return;
    }

    pub fn mouse_released(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        if mouse_info.button == self.mouse_info.button {
            v.end_modification();
            v.pop_behavior();
        }
    }
}

#[rustfmt::skip]
impl ToolBehavior for RotateImage {
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
