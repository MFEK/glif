use skulpin::skia_safe::{Canvas, Matrix};

use crate::state::Editor;

pub fn redraw_viewport(v: &Editor, canvas: &mut Canvas) {
    let mut matrix = Matrix::new_identity();
    let now_matrix = canvas.total_matrix();
    matrix.set_scale_translate((v.factor, v.factor), v.offset);

    if matrix != now_matrix {
        canvas.set_matrix(&matrix.into());
    }
}
