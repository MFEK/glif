use glifparser::{Point, PointData, WhichHandle};
use MFEKmath::polar::PolarCoordinates;

pub fn imgui_decimal_text_field(label: &str, ui: &imgui::Ui, data: &mut f32) {
    let mut x = format!("{}",(*data * 1000.).round()/1000.);
    let label = String::from(label);
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
    if entered {
        if x.as_str().len() > 0 {
            let new_x: f32 = x.as_str().parse().unwrap();
            *data = new_x;
        }
    }
}

pub fn imgui_decimal_text_field_f64(label: &str, ui: &imgui::Ui, data: &mut f64) {
    let mut x = format!("{}",(*data * 1000.).round()/1000.);
    let label = String::from(label);
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
    if entered {
        if x.as_str().len() > 0 {
            let new_x: f64 = x.as_str().parse().unwrap();
            *data = new_x;
        }
    }
}

pub fn imgui_radius_theta<PD: PointData>(
    label: &str,
    ui: &imgui::Ui,
    ar: f32,
    atheta: f32,
    wh: WhichHandle,
    point: &mut Point<PD>,
) {
    let r_label = format!("{}r", label);
    let theta_label = format!("{}θ", label);
    // Ar
    let mut ars = format!("{}", ar);
    let r_entered;
    {
        let it = ui.input_text(&r_label, &mut ars);
        r_entered = it
            .enter_returns_true(true)
            .chars_decimal(true)
            .chars_noblank(true)
            .auto_select_all(true)
            .build();
    }
    // AΘ
    let mut athetas = format!("{}", atheta);
    let theta_entered;
    {
        let it = ui.input_text(&theta_label, &mut athetas);
        theta_entered = it
            .enter_returns_true(true)
            .chars_decimal(true)
            .chars_noblank(true)
            .auto_select_all(true)
            .build();
    }
    if r_entered || theta_entered {
        let mut new_r: f32 = f32::NAN;
        if ars.as_str().len() > 0 {
            new_r = ars.as_str().parse().unwrap();
        }
        let mut new_theta: f32 = f32::NAN;
        if athetas.as_str().len() > 0 && athetas.as_str() != "NaN" {
            new_theta = athetas.as_str().parse().unwrap();
        }
        if new_r != f32::NAN && new_theta != f32::NAN {
            point.set_polar(wh, (new_r, new_theta));
        }
    }
}
