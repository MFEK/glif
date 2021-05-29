use skulpin::skia_safe::Canvas;

use crate::renderer::string::UiString;
use crate::user_interface::viewport::Viewport;

pub fn draw_point_number(viewport: &Viewport, at: (f32, f32), number: isize, canvas: &mut Canvas) {
    let converted = number.to_string();
    let uis = UiString::new(&converted).padding(10.);
    uis.draw(viewport, at, canvas);
}

pub fn draw_point_location(viewport: &Viewport, at: (f32, f32), original: (f32, f32), canvas: &mut Canvas) {
    let converted = format!("{}, {}", original.0, original.1);
    let uis = UiString::new(&converted).padding(10.);
    uis.draw(viewport, at, canvas);
}
