use crate::state::state;

pub fn calc_x(x: f32) -> f32 { x }
pub fn uncalc_x(x: f32) -> f32 { x }
// Glif format y is flipped according to Skia
pub fn calc_y(y: f32) -> f32 { 800.+(y*-1.) }
pub fn uncalc_y(y: f32) -> f32 { 800.-(y*-1.) }
