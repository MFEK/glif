use derive_more::Display;
use enum_iterator::IntoEnumIterator;

#[derive(IntoEnumIterator, Display, Debug, Clone, Copy, PartialEq)]
pub enum HandleStyle {
    None,
    Handlebars,
    Floating,
}

#[derive(Display, Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Pan,
    Pen,
    Select,
    Zoom,
    VWS,
}

#[derive(IntoEnumIterator, Display, Debug, Clone, Copy, PartialEq)]
pub enum PointLabels {
    None,
    Numbered,
    Locations,
}

#[derive(IntoEnumIterator, Debug, Clone, Copy, PartialEq)]
pub enum PreviewMode {
    None,
    NoUnselectedPoints,
    Paper,
}
