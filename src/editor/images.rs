use skulpin::skia_safe::{ColorInfo as SkColorInfo, ColorType as SkColorType, AlphaType as SkAlphaType, ColorSpace as SkColorSpace, ImageInfo as SkImageInfo};
use skulpin::skia_safe::{Image as SkImage, Data as SkData, Matrix};
use glifparser::{Color as GlifColor, Image as GpImage, image::{DataLoadState, DataOrBitmap}};

use glifparser::matrix::ToSkiaMatrix;

use std::collections::HashMap;
use std::path::PathBuf;
use std::fmt::Debug;

use super::Editor;

#[derive(Clone)]
pub struct EditorImage {
    pub img: SkImage,
    // We need to keep the data around, because SkImage doesn't own it.
    pub data: Vec<u8>,
    pub matrix: Matrix,
    pub color: Option<GlifColor>,
}
pub type EditorImages = HashMap<PathBuf, EditorImage>;

impl Debug for EditorImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "EditorImage{{img: SkImage, data: Vec<u8, length={}>, matrix: {:?}, color: {:?}}}", self.data.len(), self.matrix, self.color)
    } 
}

impl From<GpImage> for EditorImage {
    fn from(image: GpImage) -> Self {
        let (data, width, height) = match image.data.data {
            DataOrBitmap::Bitmap { pixels, width, height } => (pixels, width, height),
            // This should be unreachable if called via Editor::recache_images
            _ => panic!("You must decode the image before using it in the editor")
        };
        let color_info = SkColorInfo::new(SkColorType::RGBA8888, SkAlphaType::Premul, SkColorSpace::new_srgb());
        let image_info = SkImageInfo::from_color_info((width as i32, height as i32), color_info);
        let skdata = unsafe { SkData::new_bytes(&data) };
        let img = SkImage::from_raster_data(&image_info, skdata, (width * 4) as usize).unwrap();

        Self {
            img,
            data,
            matrix: image.matrix.to_skia_matrix(),
            color: image.color,
        }
    }
}

impl Editor {
    pub fn recache_images(&mut self) {
        self.images = self.with_glyph(|glif| {
            let mut ret = HashMap::new();
            for layer in &glif.layers {
                for (image, _matrix) in &layer.images {
                    let im = image.to_image_of(glif);
                    match im {
                        Err(_) => warn!("Failed to read image {:?}", image.filename),
                        Ok(mut gpim) => {
                            gpim.load().unwrap_or_else(|_|warn!("Failed to read image {:?}", image.filename));
                            gpim.decode().unwrap_or_else(|_|warn!("Failed to decode image {:?}", image.filename));
                            if gpim.data.state == DataLoadState::Decoded {
                                ret.insert(image.filename.clone(), gpim.into());
                            }
                        },
                    }
                }
            }
            ret
        });
    }
}
