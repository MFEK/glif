pub mod grid;

// TODO: Replace console! pub mod console;

use glifparser::matrix::ToSkiaMatrix;
use glifrenderer::anchors::draw_anchors;
use glifrenderer::calc_x;
use glifrenderer::calc_y;
use glifrenderer::glyph::draw_components;
use glifrenderer::guidelines;
use glifrenderer::points;
use glifrenderer::viewport;
use glifrenderer::constants::*;
use glifrenderer::toggles::*;

use skulpin::skia_safe::Canvas;
use skulpin::skia_safe::Matrix;

use glifparser::FlattenedGlif;
use crate::user_interface::PAPER_DRAW_GUIDELINES;
use crate::{editor::Editor, user_interface::Interface};

pub fn render_frame(v: &mut Editor, i: &mut Interface, canvas: &mut Canvas) {
    canvas.save();
    let pm = i.viewport.preview_mode;
    canvas.clear(if pm == PreviewMode::Paper {
        PAPER_BGCOLOR
    } else {
        BACKGROUND_COLOR
    });
    // This will change the SkCanvas transformation matrix, and everything from here to
    // canvas.restore() will need to take that matrix into consideration.
    viewport::redraw_viewport(&i.viewport, canvas);

    let dropped = v.with_glyph(|glif| {
        let mut dropped = vec![];
        for layer in &glif.layers {
            for (l_image, i_matrix) in &layer.images {
                if !v.images.contains_key(&l_image.filename) {
                    log::error!("Layer's image {} has gone out of scope and will be dropped! Did you save the glyph into a location without the image?", l_image.filename.to_string_lossy());
                    dropped.push(l_image.filename.clone());
                    continue;
                }
                let image = &v.images[&l_image.filename];
                let origin_transform = Matrix::translate((0., 0. - image.img.height() as f32));
                let matrix3 = Matrix::translate((calc_x(0.), calc_y(0.)));
                let tm = canvas.local_to_device_as_3x3();
                canvas.save();
                //let matrix2 = EncodedOrigin::to_matrix(EncodedOrigin::BottomLeft, (image.img.width(), image.img.height()));
                let matrix = tm * matrix3 * i_matrix.to_skia_matrix() * origin_transform ;
                canvas.set_matrix(&((matrix).into()));
                //eprintln!("{:?}", Matrix::new_identity().set_rotate(45., None).to_affine());
                // We shouldn't use (0., 0.) because this is a glifparser image, relative to the glif's points.
                // So, we need points::calc::calc_(x|y). Remember also, the glif y is positive, while Skia's
                // negative.
                canvas.draw_image(&image.img, (0., 0.), None);
                canvas.restore();
            }
        }
        dropped
    });

    for dropee in dropped {
        v.with_glyph_mut(|glif| {
            for layer in glif.layers.iter_mut() {
                layer.images.retain(|(gi, _)| gi.filename != dropee);
            }
        });
    }

    if pm != PreviewMode::Paper || PAPER_DRAW_GUIDELINES {
        v.with_glyph(|glif| {
            guidelines::draw_all(glif, &i.viewport, canvas);
        });
    }

    let active_layer = v.get_active_layer();
    let path = glifrenderer::glyph::draw(canvas, v.preview.as_ref().unwrap(), &i.viewport, active_layer);

    v.with_glyph(|glyph| {
        // Cache component rects and flattened outline on MFEKGlif
        draw_components(glyph, &i.viewport, canvas);
    });


    // TODO: let _path = glyph::draw_previews(v, canvas);

    match pm {
        PreviewMode::None => {
            let active_layer = v.get_active_layer();
            let cidx = v.contour_idx;
            let pidx = v.point_idx;
            let selected = v.selected.clone();

            v.with_glyph(|glif| {
                points::draw_all(glif, &i.viewport, active_layer, cidx, pidx, &selected, canvas);
            });
            points::draw_directions(&i.viewport, path, canvas);
            
            v.with_glyph(|glif| {
                draw_anchors(glif, &i.viewport, canvas);
            });
            //points::draw_selected(v, canvas);

            v.dispatch_tool_draw(i, canvas);
        }
        PreviewMode::NoUnselectedPoints => {
            //points::draw_selected(v, canvas);
        }
        PreviewMode::Paper => (),
    }

    if let Some(grid) = &i.grid {
        grid::draw_grid(canvas, grid, &i.viewport);
    }

    // Reset transformation matrix
    canvas.restore();

    // Draw console
    // TODO: Replace console! CONSOLE.with(|c| c.borrow_mut().draw(i, canvas));
}
