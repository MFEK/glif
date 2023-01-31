use egui::{Context, Ui};
use crate::{editor::Editor, user_interface::{Interface, gui::window::GlifWindow}, get_point};
use glifparser::{glif::{contour::MFEKContourCommon, point::MFEKPointCommon}, Point, MFEKPointData, PointData, WhichHandle, Handle};

pub struct GridWindow {
    // is this window open?
    open: bool,
}

impl GridWindow {
    pub fn new() -> Self {
        Self { open: false }
    }
}

impl GlifWindow for GridWindow {
    fn open(&self) -> bool {
        self.open
    }

    fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    fn build(&mut self, ctx: &Context, v: &mut Editor, i: &mut Interface) {
        egui::Window::new("Inspector")
        .resizable(true)
        .collapsible(true)
        .open(&mut self.open)
        .enabled(!v.is_modifying())
        .constrain(true)
        .show(ctx, |ui| {
                    /*
        let (tx, ty, tw, th) = i.get_tools_dialog_rect();

        imgui::Window::new(&imgui::ImString::new("Grid"))
            .bg_alpha(1.) // See comment on fn redraw_skia
            .flags(
                imgui::WindowFlags::NO_RESIZE
                    | imgui::WindowFlags::NO_MOVE
                    | imgui::WindowFlags::NO_COLLAPSE,
            )
            .position([tx, ty], imgui::Condition::Always)
            .size([tw, th], imgui::Condition::Always)
            .build(ui, || {
                let old_active = i.grid.show;
                let mut active = old_active;

                ui.checkbox(imgui::im_str!("Active"), &mut active);

                if !active {
                    i.grid.show = false;
                } else if !old_active && active {
                    i.grid.show = true;
                }

                if i.grid.show {
                    user_interface::util::imgui_decimal_text_field(
                        "Spacing",
                        ui,
                        &mut i.grid.spacing,
                        None,
                    );
                    user_interface::util::imgui_decimal_text_field(
                        "Offset",
                        ui,
                        &mut i.grid.offset,
                        None,
                    );

                    let old_italic = i.grid.slope.is_some();
                    let mut italic = i.grid.slope.is_some();
                    ui.checkbox(imgui::im_str!("Italic"), &mut italic);
                    if italic != old_italic && italic {
                        i.grid.slope = Some(0.5);
                    } else if italic != old_italic && !italic {
                        i.grid.slope = None;
                    }

                    if let Some(slope) = i.grid.slope {
                        let old_slope = slope;

                        let mut new_slope = slope;
                        user_interface::util::imgui_decimal_text_field(
                            "Slope",
                            ui,
                            &mut new_slope,
                            None,
                        );

                        if old_slope != new_slope {
                            i.grid.slope = Some(new_slope);
                        };

                        let old_angle =
                            (f32::to_degrees(f32::atan(slope)) * 10000.).round() / 10000.;
                        let mut new_angle = old_angle;

                        user_interface::util::imgui_decimal_text_field(
                            "Degrees",
                            ui,
                            &mut new_angle,
                            None,
                        );

                        if old_angle != new_angle {
                            i.grid.slope = Some(f32::tan(f32::to_radians(new_angle)));
                        }
                    }

                    i.grid.offset %= i.grid.spacing;
                }
            });
            */
        };
    }
}
