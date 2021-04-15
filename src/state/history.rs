use glifparser::Layer;
pub enum HistoryType {
    LayerModified,
    LayerAdded,
    LayerDeleted
}

pub struct HistoryEntry<P: glifparser::PointData> {
    // we store the cloned layer and the kind of modification
    pub description: String,
    pub layer_idx: Option<usize>,
    pub contour_idx: Option<usize>,
    pub point_idx: Option<usize>,
    pub layer: Layer<P>,
    pub kind: HistoryType

    // we also store the selection data
}