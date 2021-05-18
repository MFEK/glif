//! Skia renderer.

use crate::{tools::{EditorEvent}, editor::{PreviewMode, Editor}};
use crate::{CONSOLE};
use glifparser::Handle;
use glifparser::matrix::{Affine, ToSkiaMatrix};

pub mod constants;
use self::constants::*;

pub mod console;
mod guidelines;
pub mod points; // point drawing functions
                // This imports calc_x, etc. which transforms coordinates between .glif and Skia
pub mod string;

mod anchors;
mod glyph;
pub mod viewport;

// Provides thread-local global variables.
// TODO: pub use crate::events::vws;
pub use crate::editor::{HandleStyle, PointLabels}; // enums

use skulpin::skia_safe::{Canvas, Matrix, EncodedOrigin};

pub use points::calc::{calc_x, calc_y};

#[derive(Clone, Copy, PartialEq)]
pub enum UIPointType {
    Point((Handle, Handle)),
    Handle,
    #[allow(unused)]
    Anchor,
    Direction,
}

pub fn render_frame(v: &mut Editor, canvas: &mut Canvas) {
    canvas.save();
    let pm = v.viewport.preview_mode;
    canvas.clear(if pm == PreviewMode::Paper {
        PAPER_BGCOLOR
    } else {
        BACKGROUND_COLOR
    });
    // This will change the SkCanvas transformation matrix, and everything from here to
    // canvas.restore() will need to take that matrix into consideration.
    viewport::redraw_viewport(v, canvas);

    v.with_glyph(|glif| {
        for layer in &glif.layers {
            for (l_image, i_matrix) in &layer.images {
                let image = &v.images[&l_image.filename];
                let origin_transform = Matrix::translate((0., 0. - image.img.height() as f32));
                let matrix3 = Matrix::translate((calc_x(0.), calc_y(0.)));
                let mut tm = canvas.local_to_device_as_3x3();
                canvas.save();
                //let matrix2 = EncodedOrigin::to_matrix(EncodedOrigin::BottomLeft, (image.img.width(), image.img.height()));
                let mut matrix = tm * matrix3 * i_matrix.to_skia_matrix() * origin_transform ;
                canvas.set_matrix(&((matrix).into()));
                //eprintln!("{:?}", Matrix::new_identity().set_rotate(45., None).to_affine());
                // We shouldn't use (0., 0.) because this is a glifparser image, relative to the glif's points.
                // So, we need points::calc::calc_(x|y). Remember also, the glif y is positive, while Skia's
                // negative.
                canvas.draw_image(&image.img, (0., 0.), None);
                canvas.restore();
            }
        }
    });

    if pm != PreviewMode::Paper || PAPER_DRAW_GUIDELINES {
        guidelines::draw_all(v, canvas);
    }

    let active_layer = v.get_active_layer();
    let path = glyph::draw(canvas, v, active_layer);

    // TODO: let _path = glyph::draw_previews(v, canvas);

    match pm {
        PreviewMode::None => {
            points::draw_all(v, canvas);
            points::draw_directions(v, path, canvas);
            anchors::draw_anchors(v, canvas);
            //points::draw_selected(v, canvas);
            v.dispatch_editor_event(EditorEvent::Draw {
                skia_canvas: canvas,
            });
        }
        PreviewMode::NoUnselectedPoints => {
            //points::draw_selected(v, canvas);
        }
        PreviewMode::Paper => (),
    }
    // Reset transformation matrix
    canvas.restore();

    // Draw console
    CONSOLE.with(|c| c.borrow_mut().draw(v, canvas));
}
