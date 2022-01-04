use glifparser::{Point, PointData, WhichHandle};
use MFEKmath::polar::PolarCoordinates;

pub fn imgui_decimal_text_field(
    label: &str,
    ui: &imgui::Ui,
    data: &mut f32,
    f: Option<Box<dyn for<'a, 'b> FnMut(imgui::InputText<'a, 'b>) -> imgui::InputText<'a, 'b>>>,
) {
    let mut x = imgui::im_str!("{}", (*data * 1000.).round() / 1000.);
    let label = imgui::ImString::new(label);
    let tok = ui.push_item_width(100.);
    let entered;
    {
        let mut it = ui.input_text(&label, &mut x);
        it = it
            .enter_returns_true(true)
            .chars_decimal(true)
            .chars_noblank(true)
            .auto_select_all(true);
        if let Some(mut func) = f {
            it = func(it);
        }
        entered = it.build();
    }
    if entered && !x.to_str().is_empty() {
        let new_x: f32 = x.to_str().parse().unwrap();
        *data = new_x;
    }
    tok.pop(ui);
}

pub fn imgui_decimal_text_field_f64(label: &str, ui: &imgui::Ui, data: &mut f64) {
    let mut x = imgui::im_str!("{}", (*data * 1000.).round() / 1000.);
    let label = imgui::ImString::new(label);
    let entered;
    {
        let it = ui.input_text(&label, &mut x);
        entered = it
            .enter_returns_true(true)
            .chars_decimal(true)
            .chars_noblank(true)
            .auto_select_all(true)
            .build();
    }
    if entered && !x.to_str().is_empty() {
        let new_x: f64 = x.to_str().parse().unwrap();
        *data = new_x;
    }
}

pub fn imgui_radius_theta<PD: PointData>(
    label: &str,
    ui: &imgui::Ui,
    wh: WhichHandle,
    point: &mut Point<PD>,
) {
    let (radius, mut theta) = point.polar(wh);
    theta = theta.to_degrees();
    theta -= 180.;
    if theta.is_sign_positive() && wh == WhichHandle::B {
        theta = 360. - theta;
    }
    let r_label = imgui::im_str!("{}r", label);
    let theta_label = imgui::im_str!("{}θ", label);
    // Ar
    let mut radii = imgui::im_str!("{}", radius);
    let r_entered;
    {
        let it = ui.input_text(&r_label, &mut radii);
        r_entered = it
            .enter_returns_true(true)
            .chars_decimal(true)
            .chars_noblank(true)
            .auto_select_all(true)
            .build();
    }
    // AΘ
    let mut thetas = imgui::im_str!("{}", theta);
    let theta_entered;
    {
        let it = ui.input_text(&theta_label, &mut thetas);
        theta_entered = it
            .enter_returns_true(true)
            .chars_decimal(true)
            .chars_noblank(true)
            .auto_select_all(true)
            .build();
    }
    if r_entered || theta_entered {
        let mut new_r: f32 = f32::NAN;
        if radii.to_str().len() > 0 {
            new_r = radii.to_str().parse().unwrap();
        }
        let mut new_theta: f32 = f32::NAN;
        if thetas.to_str().len() > 0 && thetas.to_str() != "NaN" {
            new_theta = thetas.to_str().parse().unwrap();
        }
        if !f32::is_nan(new_r) && !f32::is_nan(new_theta) {
            point.set_polar(wh, (new_r, new_theta));
        }
    }
}
