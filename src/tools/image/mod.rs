use std::convert::TryInto;

use crate::command::CommandMod;
use crate::editor::Editor;
use crate::user_interface::{Interface, MouseInfo};
use super::prelude::*;
use MFEKmath::{Vector};
use glifparser::matrix::ToSkiaMatrix;
use crate::filedialog;
use skulpin::skia_safe::{Paint, Path, PaintStyle};
use kurbo::Affine;

mod dialog;

// The image tool is for adding and manipulating images on layers. With the image tool selected you can click an empty space in order
// to add an image to the current layer. Clicking an image without holding any modifiers translates that image. Clicking while holding
// ctrl rotates the image around it's center.
#[derive(Clone)]
pub struct Image {
    selected_idx: Option<usize>,
    last_pos: Option<(f32, f32)>,
    debug: MouseInfo,
    pivot_point: Option<(f32, f32)>,
    rotate_vector: Option<(f32, f32)>,
}

impl Tool for Image {
    fn handle_event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent { event_type, meta } => {
                self.debug = meta;
                match event_type {
                    MouseEventType::Moved => { self.mouse_moved(v, i, meta) }
                    MouseEventType::Pressed => { self.mouse_pressed(v, meta) }
                    MouseEventType::Released => { self.mouse_released(v) }
                    _ => {}
                }
            }
            EditorEvent::Draw { skia_canvas } => { 
                self.draw_selected(v, i, skia_canvas);
            }
            EditorEvent::Ui { ui } => {
                self.tool_dialog(v, i, ui);
            }
            _ => {}
        }
    }
}

// Here you can implement behaviors for events. 
impl Image {
    pub fn new() -> Self {
        Self {
            selected_idx: None,
            debug: MouseInfo { // this really needs a default
                button: sdl2::mouse::MouseButton::Left,
                position: (0., 0.),
                raw_position: (0., 0.),
                absolute_position: (0., 0.),
                raw_absolute_position: (0., 0.),
                is_down: (false),
                modifiers: CommandMod{
                    shift: false,
                    ctrl: false,
                    alt: false,
                },
            },
            last_pos: None,
            rotate_vector: None,
            pivot_point: None,
        }
    }

    fn is_image_clicked(&self, v: &Editor, meta: MouseInfo) -> Option<usize> {
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

                let f_rect = SkRect::new(img_rect.left() as f32, img_rect.top() as f32, img_rect.right() as f32, img_rect.bottom() as f32);
                let final_img_rect = origin_mat.map_rect(f_rect).0;

                let local_mouse = SkPoint::new(meta.position.0, meta.position.1);

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

            let f_rect = SkRect::new(img_rect.left() as f32, img_rect.top() as f32, img_rect.right() as f32, img_rect.bottom() as f32);
            origin_mat.map_rect(f_rect).0
        })
    }

    fn mouse_moved(&mut self, v: &mut Editor, _i: &mut Interface, meta: MouseInfo) {
        // when the user has their mouse down we should translate the image by the mouse delta
        if meta.is_down {
            if let Some(selected) = self.selected_idx {
                if let Some(lp) = self.last_pos {
                    if let Some(rot) = self.rotate_vector {
                        let pivot = self.pivot_point.unwrap();
                        let pivot_vector = Vector::from_components(pivot.0 as f64, pivot.1 as f64);
                        let mouse_vector = Vector::from_components(meta.position.0 as f64, meta.position.1 as f64);
                    
                        let normal_from_pivot = (pivot_vector - mouse_vector).normalize();
                
                        let rot_vec = Vector::from_components(rot.0 as f64, rot.1 as f64);
                        let rotation_angle = normal_from_pivot.angle(rot_vec);

                        self.rotate_vector = Some(normal_from_pivot.to_tuple());
                        
                        v.with_active_layer_mut(|layer| {
                            let affine = layer.images[selected].1.clone();
                            let raw_affine: Vec<f32> = affine.as_coeffs().iter().map(|x| *x as f32).collect();
    
                            let sk_affine = Matrix::from_affine(&raw_affine.try_into().unwrap());
                            let rotate_mat = Matrix::rotate_rad(-rotation_angle as f32);
    
                            let sk_affine = sk_affine * rotate_mat;
    
                            let translated_raw_affine = sk_affine.to_affine();
    
                            if let Some(tra) = translated_raw_affine {
                                let tra: Vec<f64> = tra.iter().map(|x| *x as f64).collect();
                                layer.images[selected].1 = Affine::new(tra.try_into().unwrap());
                            }
                        });

                        return;
                    }

                    let dx = meta.position.0 - lp.0;
                    let dy = meta.position.1 - lp.1;

                    self.last_pos = Some(meta.position);
    
                    v.with_active_layer_mut(|layer| {
                        let affine = layer.images[selected].1.clone();
                        let raw_affine: Vec<f32> = affine.as_coeffs().iter().map(|x| *x as f32).collect();

                        let sk_affine = Matrix::from_affine(&raw_affine.try_into().unwrap());
                        let translate_mat = Matrix::translate((dx, dy));

                        let sk_affine = translate_mat * sk_affine;

                        let translated_raw_affine = sk_affine.to_affine();

                        if let Some(tra) = translated_raw_affine {
                            let tra: Vec<f64> = tra.iter().map(|x| *x as f64).collect();
                            layer.images[selected].1 = Affine::new(tra.try_into().unwrap());
                        }
                    })
                } else {
                    self.last_pos = Some((meta.position.0, meta.position.1));
                    v.begin_layer_modification("Translate image.");
                }
            }
        }
    }

    // When the mouse is pressed we store the point.
    fn mouse_pressed(&mut self, v: &mut Editor,  meta: MouseInfo) {
        // if we did click an image we're going to want to let the user translate/rotate that image
        if let Some(img_idx) = self.is_image_clicked(v, meta) {
            self.selected_idx = Some(img_idx);

            if meta.modifiers.ctrl {
                let pivot = self.get_image_pivot(v, img_idx);
                let pivot_vector = Vector::from_components(pivot.0 as f64, pivot.1 as f64);
                let mouse_vector = Vector::from_components(meta.position.0 as f64, meta.position.1 as f64);
            
                self.pivot_point = Some(pivot);
                self.rotate_vector = Some((pivot_vector - mouse_vector).normalize().to_tuple());
            }
        } else {
            // we should clear the selected image here
            self.selected_idx = None;
            self.rotate_vector = None;

            // if we didn't click an image we're going to open a prompt to select one from the file
            // system
            let filename = match filedialog::open_image(None) {
                Some(f) => f,
                None => return,
            };

            v.begin_layer_modification("Add image to layer.");
            v.add_image_to_active_layer(filename);
            v.end_layer_modification();
            return;
        }

        
        // if the user held control then we want to allow them to rotate that image similar to how point rotation works

    }

    fn draw_selected(&mut self, v: &Editor, _i: &Interface, canvas: &mut Canvas) {
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

    // When it's released we set it to none.
    fn mouse_released(&mut self, v: &mut Editor) {
        self.last_pos = None;
        self.rotate_vector = None;
        self.pivot_point = None;
        v.end_layer_modification();
    }
}
