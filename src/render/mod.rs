// TODO: Replace console! pub mod console;

use glifparser::matrix::ToSkiaMatrix;
use glifrenderer::anchors::draw_anchors;
use glifrenderer::constants::*;
use glifrenderer::glyph::draw_components;
use glifrenderer::grid;
use glifrenderer::guidelines;
use glifrenderer::points;
use glifrenderer::toggles::*;

use skia_safe::{self as skia, Canvas};

use crate::user_interface::PAPER_DRAW_GUIDELINES;
use crate::{editor::Editor, user_interface::Interface};

mod speed_visualization;
pub mod measure;

pub fn render_frame(v: &mut Editor, i: &mut Interface, canvas: &Canvas) {
    canvas.save();

    let pm = i.viewport.preview_mode;
    canvas.clear(if pm == PreviewMode::Paper {
        PAPER_BGCOLOR
    } else {
        BACKGROUND_COLOR
    });

    // This will change the SkCanvas transformation matrix, and everything from here to
    // canvas.restore() will need to take that matrix into consideration.
    i.viewport.redraw(canvas);

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
                let tm = canvas.local_to_device_as_3x3();
                canvas.save();
                let matrix2 = skia::EncodedOrigin::to_matrix(skia::EncodedOrigin::BottomLeft, (image.img.width(), image.img.height()));
                let matrix = tm * i_matrix.to_skia_matrix() * matrix2;
                canvas.set_matrix(&((matrix).into()));
                //eprintln!("{:?}", Matrix::new_identity().set_rotate(45., None).to_affine());
                canvas.draw_image(&image.img, (0., 0.), None);
                canvas.restore();
            }
        }
        dropped
    });

    for dropee in dropped {
        v.with_glyph_mut_no_history(|glif| {
            for layer in glif.layers.iter_mut() {
                layer.images.retain(|(gi, _)| gi.filename != dropee);
            }
        });
    }

    if pm != PreviewMode::Paper || PAPER_DRAW_GUIDELINES {
        guidelines::draw_baseline::<()>(&i.viewport, canvas);
        let local_guidelines = v.with_glyph(|glyph| {
            glyph
                .guidelines
                .iter()
                .map(|g| g.clone())
                .collect::<Vec<_>>()
        });
        for guideline in v.guidelines.iter().chain(local_guidelines.iter()) {
            let data = guideline.data.as_guideline();
            guidelines::draw_guideline(
                &i.viewport,
                canvas,
                &guideline,
                if data.right {
                    Some(RBEARING_STROKE)
                } else if data.format {
                    Some(UFO_GUIDELINE_STROKE)
                } else {
                    None
                },
            );
        }
        if i.grid.show {
            grid::draw(canvas, &i.grid, &i.viewport);
        }
    }

    glifrenderer::glyph::draw(canvas, v.preview.as_ref().unwrap(), &i.viewport, None);

    speed_visualization::draw_velocity(v, i, canvas);
    v.with_glyph(|glyph| {
        // Cache component rects and flattened outline on MFEKGlif
        draw_components(glyph, &i.viewport, canvas);
    });
    i.measure.draw_line(i, v, canvas, i.viewport.factor);

    // TODO: let _path = glyph::draw_previews(v, canvas);

    match pm {
        PreviewMode::None | PreviewMode::NoUnselectedPoints => {
            let active_layer = v.get_active_layer();
            let cidx = v.contour_idx;
            let pidx = v.point_idx;
            let selected = v.selected.clone();

            v.with_glyph(|glif| {
                points::draw_all(
                    glif,
                    &i.viewport,
                    active_layer,
                    cidx,
                    pidx,
                    &selected,
                    canvas,
                    pm != PreviewMode::None,
                );
                draw_anchors(glif, &i.viewport, canvas);
            });

            points::draw_directions(
                &i.viewport,
                v.get_active_layer_ref(),
                canvas,
                &selected,
                pm != PreviewMode::None,
            );

            v.dispatch_tool_draw(i, canvas);
        }
        PreviewMode::Paper => (),
    }

    // Reset transformation matrix
    canvas.restore();

    // Draw console
    // TODO: Replace console! CONSOLE.with(|c| c.borrow_mut().draw(i, canvas));
}
