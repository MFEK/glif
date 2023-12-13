use crate::contour_operations::ContourOperationBuild;
use crate::user_interface::Interface;
use glifparser::glif::contour::MFEKContourCommon;
use glifparser::outline::skia::{FromSkiaPath, ToSkiaPaths};
use glifparser::{
    glif::{Layer, LayerOperation},
    MFEKGlif, Outline,
};
use glifparser::{FlattenedGlif, MFEKPointData};
use skia_safe::PathOp;
use MFEKmath::mfek::ResolveCubic;

use super::Editor;

impl Editor {
    pub fn mark_preview_dirty(&mut self) {
        self.preview_dirty = true;
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn rebuild(&mut self, i: &mut Interface) {
        self.handle_filesystem_events(i);

        if !self.preview_dirty {
            return;
        };

        if self.glyph.as_ref().unwrap().layers[0].operation.is_some() {
            self.glyph.as_mut().unwrap().layers[0].operation = None;
        }

        //self.fix_contour_ops();
        let mut preview_layers = Vec::new();
        for layer in &self.glyph.as_mut().unwrap().layers {
            let mut preview_outline = Vec::new();

            for (_idx, glif_contour) in layer.outline.iter().enumerate() {
                if glif_contour.inner().len() <= 1 {
                    preview_outline.push(glif_contour.to_cubic());
                    continue;
                }

                let build_result: Vec<glifparser::glif::MFEKContour<MFEKPointData>> =
                    glif_contour.operation().build(glif_contour);

                for new_contour in build_result {
                    preview_outline.push(new_contour.to_cubic());
                }
            }

            let mut new_layer = layer.clone();
            new_layer.outline = preview_outline;
            preview_layers.push(new_layer);
        }

        let mut rects = Some(vec![]);
        let flattened = self.glyph.as_mut().unwrap().flattened(&mut rects);
        flattened
            .map(|f| {
                self.glyph.as_mut().unwrap().flattened = f.flattened;
                self.glyph.as_mut().unwrap().component_rects = rects;
            })
            .unwrap_or_else(|e| log::error!("Failed to draw components: {:?}", e));

        self.preview = Some(self.glyph.as_ref().unwrap().clone());
        self.preview.as_mut().unwrap().layers = preview_layers;
        self.preview_dirty = false;
    }

    pub fn prepare_export(&self) -> MFEKGlif<MFEKPointData> {
        let glyph = self
            .glyph
            .as_ref()
            .expect("Illegally tried to export a null glyph!");
        if glyph.layers.len() == 1
            && glyph.layers[0]
                .outline
                .iter()
                .all(|c| c.operation().clone() == None)
        {
            return glyph.clone();
        }

        let glif = self.preview.as_ref().unwrap_or(glyph);

        // MFEKGlif always has a layer zero so this is safe. (No it isn't, it can be invisible. TODO: Fix this.)
        let mut last_combine_layer: Layer<MFEKPointData> = glif.layers[0].clone();
        let mut exported_layers: Vec<Layer<MFEKPointData>> = vec![];
        let mut current_layer_group = last_combine_layer.outline.to_skia_paths(None).combined();

        for (layer_idx, layer) in glif.layers.iter().enumerate() {
            if !layer.visible {
                continue;
            }
            if layer_idx == 0 {
                continue;
            }

            let skpaths = layer.outline.to_skia_paths(None);

            match &layer.operation {
                Some(op) => {
                    let pathop = match op {
                        LayerOperation::Difference => PathOp::Difference,
                        LayerOperation::Union => PathOp::Union,
                        LayerOperation::Intersect => PathOp::Intersect,
                        LayerOperation::XOR => PathOp::XOR,
                    };

                    if let Some(result) = current_layer_group
                        .op(&(skpaths.combined()), pathop)
                        .unwrap()
                        .as_winding()
                    {
                        current_layer_group = result;
                    }
                }

                None => {
                    let mut combined_layer = last_combine_layer.clone();
                    last_combine_layer = layer.clone();

                    let combined_layer_outline = Outline::from_skia_path(&current_layer_group);
                    let mfek_outline = combined_layer_outline.iter().map(|c| c.into()).collect();
                    combined_layer.outline = mfek_outline;
                    exported_layers.push(combined_layer);

                    current_layer_group = layer.outline.to_skia_paths(None).combined();
                }
            }
        }

        let mut combined_layer = last_combine_layer;
        let combined_layer_outline = Outline::from_skia_path(&current_layer_group);
        let mfek_outline = combined_layer_outline.iter().map(|c| c.into()).collect();
        combined_layer.outline = mfek_outline;
        exported_layers.push(combined_layer);

        let mut exported_mfek = glif.clone();
        exported_mfek.layers = exported_layers;
        exported_mfek
    }
}
