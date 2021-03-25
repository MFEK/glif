use crate::STATE;
use skulpin::skia_safe::{Canvas, Matrix};

pub fn redraw_viewport(canvas: &mut Canvas) {
    let scale = STATE.with(|v| v.borrow().factor);
    let offset = STATE.with(|v| v.borrow().offset);
    let mut matrix = Matrix::new_identity();
    matrix.set_scale_translate((scale, scale), offset);

    canvas.set_matrix(&matrix.into());
}
