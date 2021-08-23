use glifparser::outline::skia::{FromSkiaPath, ToSkiaPaths};
use glifparser::{
    glif::{Layer, LayerOperation, MFEKPointData},
    MFEKGlif, Outline,
};
use skulpin::skia_safe::{Path, PathOp};

use crate::contour_operations;

use super::Editor;

impl Editor {
    pub fn mark_preview_dirty(&mut self) {
        self.preview_dirty = true;
    }

    pub fn rebuild(&mut self) {
        if !self.preview_dirty {
            return;
        };

        if self.glyph.as_ref().unwrap().layers[0].operation.is_some() {
            self.glyph.as_mut().unwrap().layers[0].operation = None;
        }

        //self.fix_contour_ops();
        let mut preview_layers = Vec::new();
        for layer in &self.glyph.as_ref().unwrap().layers {
            let mut preview_outline = Vec::new();

            for (_idx, glif_contour) in layer.outline.iter().enumerate() {
                if glif_contour.inner.len() < 2 {
                    preview_outline.push(glif_contour.clone());
                    continue;
                }

                let build_result = contour_operations::build(glif_contour);

                for new_contour in build_result {
                    preview_outline.push(new_contour);
                }
            }

            let mut new_layer = layer.clone();
            new_layer.outline = preview_outline;
            preview_layers.push(new_layer);
        }

        self.preview = Some(self.glyph.as_ref().unwrap().clone());
        self.preview.as_mut().unwrap().layers = preview_layers;
        self.preview_dirty = false;
    }

    pub fn prepare_export(&self) -> MFEKGlif<MFEKPointData> {
        let glif = self.preview.as_ref().unwrap();

        // MFEKGlif always has a layer zero so this is safe. (No it isn't, it can be invisible. TODO: Fix this.)
        let mut last_combine_layer: Layer<MFEKPointData> = glif.layers[0].clone();
        let mut exported_layers: Vec<Layer<MFEKPointData>> = vec![];
        let new_combine_paths = last_combine_layer.outline.to_skia_paths(None).closed;
        let mut current_layer_group = new_combine_paths.unwrap_or(Path::new());

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

                    if let Some(closed) = skpaths.closed {
                        if let Some(result) = current_layer_group
                            .op(&closed, pathop)
                            .unwrap()
                            .as_winding()
                        {
                            current_layer_group = result;
                        }
                    }
                }

                None => {
                    let mut combined_layer = last_combine_layer.clone();
                    last_combine_layer = layer.clone();

                    let combined_layer_outline = Outline::from_skia_path(&current_layer_group);
                    let mfek_outline = combined_layer_outline.iter().map(|c| c.into()).collect();
                    combined_layer.outline = mfek_outline;
                    exported_layers.push(combined_layer);

                    let new_combine_paths = layer.outline.to_skia_paths(None).closed;
                    current_layer_group = new_combine_paths.unwrap_or(Path::new());
                }
            }
        }

        let mut combined_layer = last_combine_layer.clone();
        let combined_layer_outline = Outline::from_skia_path(&current_layer_group);
        let mfek_outline = combined_layer_outline.iter().map(|c| c.into()).collect();
        combined_layer.outline = mfek_outline;
        exported_layers.push(combined_layer);

        let mut exported_mfek = glif.clone();
        exported_mfek.layers = exported_layers;
        return exported_mfek;
    }
}
