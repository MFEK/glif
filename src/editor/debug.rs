use super::Editor;

use glifparser::{glif::{LayerOperation}, outline::skia::{ToSkiaPaths, SkiaPointTransforms}};

impl Editor {
    /// Dumps the current layers to console with skpath.dump(). This is useful for debugging
    /// and creating skfiddles.
    pub fn skia_dump(&self) {
        self.with_glyph(|glif| {
            for layer in &glif.layers {
                if let Some(closed_path) = layer.outline.to_skia_paths(None).closed {
                    println!("Dumping layer named: {0}", layer.name);
                    closed_path.dump();
                }
            }
        })
    }
}