use crate::user_interface::Interface;
use skulpin::skia_safe::{Canvas, Matrix};

pub fn redraw_viewport(i: &Interface, canvas: &mut Canvas) {
    let mut matrix = Matrix::new_identity();
    let now_matrix = canvas.local_to_device_as_3x3();
    matrix.set_scale_translate((i.viewport.factor, i.viewport.factor), i.viewport.offset);

    if matrix != now_matrix {
        canvas.set_matrix(&matrix.into());
    }
}
