use crate::user_interface::{Interface, gui::windows::egui_parsed_textfield};

use super::{ShapeType, Shapes};

impl Shapes {
    pub fn shape_settings(&mut self, i: &mut Interface, ui: &mut egui::Ui) {
        ui.radio_value(&mut self.stype, ShapeType::Circle, "Circle");
        ui.radio_value(&mut self.stype, ShapeType::Oval, "Oval");
        ui.radio_value(&mut self.stype, ShapeType::Rectangle, "Rectangle");
        ui.radio_value(&mut self.stype, ShapeType::RoundedRectangle, "RoundedRectangle");
        ui.radio_value(&mut self.stype, ShapeType::Polygon, "Polygon");

        ui.separator();

        match self.stype {
            ShapeType::RoundedRectangle => {
                ui.label("Roundness:");
                ui.add(egui::Slider::new(&mut self.sdata.rrect_radius, 1f32..=200f32));
            }
            ShapeType::Polygon | ShapeType::Star => {
                ui.label("Sides");
                ui.add(egui::Slider::new(&mut self.sdata.polygon_sides, 3u16..=50u16));

                ui.label("Roundness");
                ui.add(egui::Slider::new(&mut self.sdata.polygon_radius, 0f32..=1000f32));

                ui.label("Angle");
                self.sdata.polygon_angle = egui_parsed_textfield(ui, "angle", self.sdata.polygon_angle, &mut self.edit_buf)
            }
            _ => (),
        }
    }
}