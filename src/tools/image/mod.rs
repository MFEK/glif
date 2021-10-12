use super::prelude::*;
use crate::editor::Editor;
use crate::filedialog;
use crate::tool_behaviors::move_image::MoveImage;
use crate::tool_behaviors::rotate_image::RotateImage;
use crate::user_interface::{Interface, MouseInfo};
use glifparser::matrix::ToSkiaMatrix;
use skulpin::skia_safe::{Paint, PaintStyle, Path};

mod dialog;

// The image tool is for adding and manipulating images on layers. With the image tool selected you can click an empty space in order
// to add an image to the current layer. Clicking an image without holding any modifiers translates that image. Clicking while holding
// ctrl rotates the image around it's center.
#[derive(Clone)]
pub struct Image {
    selected_idx: Option<usize>,
}

impl Tool for Image {
    fn event(&mut self, v: &mut Editor, _i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent {
                event_type,
                mouse_info,
            } => {
                match event_type {
                    MouseEventType::Pressed => self.mouse_pressed(v, mouse_info),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn draw(&self, v: &Editor, _i: &Interface, canvas: &mut Canvas) {
        if let Some(selected) = self.selected_idx {
            let mut selected_path = Path::new();
            let img_rect = self.get_image_rect(v, selected);
            selected_path.add_rect(img_rect, None);

            let mut paint = Paint::default();
            paint.set_style(PaintStyle::Stroke);
            paint.set_color(SELECTED_STROKE);

            canvas.draw_path(&selected_path, &paint);
        }
    }

    fn ui(&mut self, v: &mut Editor, i: &mut Interface, ui: &mut Ui) {
        self.tool_dialog(v, i, ui)
    }
}

// Here you can implement behaviors for events.
impl Image {
    pub fn new() -> Self {
        Self {
            selected_idx: None,
        }
    }

    fn is_image_clicked(&self, v: &Editor, mouse_info: MouseInfo) -> Option<usize> {
        v.with_active_layer(|layer| {
            // we've got to take our current mouse position and translate that into 'image space' such that the mouse position
            // is relative to 0,0 and the image forms an axis aligned bounding box
            let iter = layer.images.iter().enumerate();
            for (idx, (l_image, i_matrix)) in iter {
                let image = &v.images[&l_image.filename];

                let img_rect = image.img.bounds();

                let origin_transform = Matrix::translate((0., 0. - image.img.height() as f32));
                let matrix3 = Matrix::translate((calc_x(0.), calc_y(0.)));
                let origin_mat = matrix3 * i_matrix.to_skia_matrix() * origin_transform;

                let f_rect = SkRect::new(
                    img_rect.left() as f32,
                    img_rect.top() as f32,
                    img_rect.right() as f32,
                    img_rect.bottom() as f32,
                );
                let final_img_rect = origin_mat.map_rect(f_rect).0;

                let local_mouse = SkPoint::new(mouse_info.position.0, mouse_info.position.1);

                if final_img_rect.contains(local_mouse) {
                    return Some(idx);
                }
            }

            None
        })
    }

    fn get_image_pivot(&self, v: &Editor, idx: usize) -> (f32, f32) {
        v.with_active_layer(|layer| {
            let image = &v.images[&layer.images[idx].0.filename];

            let img_rect = image.img.bounds();

            let origin_transform = Matrix::translate((0., 0. - image.img.height() as f32));
            let matrix3 = Matrix::translate((calc_x(0.), calc_y(0.)));
            let origin_mat = matrix3 * layer.images[idx].1.to_skia_matrix() * origin_transform;
            let final_img_pivot = origin_mat.map_xy(0., img_rect.bottom() as f32);

            (final_img_pivot.x, final_img_pivot.y)
        })
    }

    fn get_image_rect(&self, v: &Editor, idx: usize) -> SkRect {
        v.with_active_layer(|layer| {
            let image = &v.images[&layer.images[idx].0.filename];

            let img_rect = image.img.bounds();

            let origin_transform = Matrix::translate((0., 0. - image.img.height() as f32));
            let matrix3 = Matrix::translate((calc_x(0.), calc_y(0.)));
            let origin_mat = matrix3 * layer.images[idx].1.to_skia_matrix() * origin_transform;

            let f_rect = SkRect::new(
                img_rect.left() as f32,
                img_rect.top() as f32,
                img_rect.right() as f32,
                img_rect.bottom() as f32,
            );
            origin_mat.map_rect(f_rect).0
        })
    }

    fn mouse_pressed(&mut self, v: &mut Editor, mouse_info: MouseInfo) {
        // if we did click an image we're going to want to let the user translate/rotate that image
        if let Some(img_idx) = self.is_image_clicked(v, mouse_info) {
            self.selected_idx = Some(img_idx);

            if mouse_info.modifiers.ctrl {
                let pivot = self.get_image_pivot(v, img_idx);
                v.set_behavior(Box::new(RotateImage::new(img_idx, pivot, mouse_info)));
                return;
            }

            v.set_behavior(Box::new(MoveImage::new(img_idx, mouse_info)))
        } else {
            // we should clear the selected image here
            self.selected_idx = None;

            // if we didn't click an image we're going to open a prompt to select one from the file
            // system
            let filename = match filedialog::open_image(None) {
                Some(f) => f,
                None => return,
            };

            v.begin_modification("Add image to layer.");
            v.add_image_to_active_layer(filename);
            v.end_modification();
            return;
        }
    }
}
