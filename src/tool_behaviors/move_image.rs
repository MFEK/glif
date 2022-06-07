use std::convert::TryInto;

use kurbo::Affine;

use super::prelude::*;

#[derive(Clone, Debug)]
pub struct MoveImage {
    selected_idx: usize,
    mouse_info: MouseInfo,
    last_position: (f32, f32),
}

impl MoveImage {
    pub fn new(selected_idx: usize, mouse_info: MouseInfo) -> Self {
        MoveImage {
            selected_idx,
            mouse_info,
            last_position: mouse_info.position,
        }
    }

    pub fn mouse_moved(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        if !v.is_modifying() {
            v.begin_modification("Move image.");
        }

        let dx = mouse_info.position.0 - self.last_position.0;
        let dy = mouse_info.position.1 - self.last_position.1;

        self.mouse_info = mouse_info;

        let affine = v.get_active_layer_mut().images[self.selected_idx].1;
        let raw_affine: Vec<f32> = affine.as_coeffs().iter().map(|x| *x as f32).collect();

        let sk_affine = Matrix::from_affine(&raw_affine.try_into().unwrap());
        let translate_mat = Matrix::translate((dx, dy));

        let sk_affine = translate_mat * sk_affine;

        let translated_raw_affine = sk_affine.to_affine();

        if let Some(tra) = translated_raw_affine {
            let tra: Vec<f64> = tra.iter().map(|x| *x as f64).collect();
            v.get_active_layer_mut().images[self.selected_idx].1 =
                Affine::new(tra.try_into().unwrap());
        }

        self.last_position = mouse_info.position;
    }

    pub fn mouse_released(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        if mouse_info.button == self.mouse_info.button {
            v.end_modification();
            v.pop_behavior();
        }
    }
}

#[rustfmt::skip]
impl ToolBehavior for MoveImage {
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
