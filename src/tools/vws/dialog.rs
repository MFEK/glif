use crate::user_interface::Interface;
use glifparser::glif::{CapType, ContourOperations, JoinType};
use super::VWS;
use super::super::prelude::*;

fn join_type_to_idx(jt: JoinType) -> usize {
    match jt {
        JoinType::Round => 0,
        JoinType::Miter => 1,
        JoinType::Bevel => 2,
        JoinType::Circle => 3,
    }
}

fn idx_to_join_type(idx: usize) -> JoinType {
    match idx {
        0 => JoinType::Round,
        1 => JoinType::Miter,
        2 => JoinType::Bevel,
        _ => unreachable!(),
    }
}

fn cap_type_to_idx(ct: CapType) -> usize {
    match ct {
        CapType::Round => 0,
        CapType::Square => 1,
        CapType::Custom => 2,
        CapType::Circle => 3,
    }
}

fn idx_to_cap_type(idx: usize) -> CapType {
    match idx {
        0 => CapType::Round,
        1 => CapType::Square,
        2 => CapType::Custom,
        _ => unreachable!(),
    }
}

impl VWS {
    fn build_and_check_vws_cap_combo(&self, v: &mut Editor, ui: &imgui::Ui) {
        let contour_idx = v.contour_idx.unwrap();

        let _vws_contour = self.get_vws_contour(v, contour_idx);

        if let Some(vws_contour) = _vws_contour {
            let old_s = cap_type_to_idx(vws_contour.cap_start_type);
            let old_e = cap_type_to_idx(vws_contour.cap_end_type);
            let mut s_current_selection = old_s;
            let mut e_current_selection = old_e;

            let options = [
                imgui::im_str!("Round"),
                imgui::im_str!("Square"),
                imgui::im_str!("Custom"),
            ];

            imgui::ComboBox::new(imgui::im_str!("Start")).build_simple_string(
                ui,
                &mut s_current_selection,
                &options,
            );

            imgui::ComboBox::new(imgui::im_str!("End")).build_simple_string(
                ui,
                &mut e_current_selection,
                &options,
            );

            let s_selection = idx_to_cap_type(s_current_selection);
            let e_selection = idx_to_cap_type(e_current_selection);

            // we only update the contour and previews when our selection changes
            if old_s != s_current_selection || e_current_selection != old_e {
                let mut new_data = vws_contour.clone();
                new_data.cap_start_type = s_selection;
                new_data.cap_end_type = e_selection;

                let new_op = ContourOperations::VariableWidthStroke { data: new_data };

                v.begin_layer_modification("VWS dialog modification.");
                v.with_active_layer_mut(|layer| layer.outline[contour_idx].operation = Some(new_op.clone()) );
                v.end_layer_modification();
            }
        }
    }

    fn build_and_check_vws_join_combo(&self, v: &mut Editor,  ui: &imgui::Ui) {
        let contour_idx = v.contour_idx.unwrap();

        let _vws_contour = self.get_vws_contour(v, contour_idx);

        if let Some(vws_contour) = _vws_contour {
            let mut current_selection = join_type_to_idx(vws_contour.join_type);

            let options = [
                imgui::im_str!("Round"),
                imgui::im_str!("Miter"),
                imgui::im_str!("Bevel"),
            ];

            imgui::ComboBox::new(imgui::im_str!("Joins")).build_simple_string(
                ui,
                &mut current_selection,
                &options,
            );

            let new_selection = idx_to_join_type(current_selection);
            if new_selection != vws_contour.join_type{
                let mut new_data = vws_contour.clone();
                new_data.join_type = new_selection;
                let new_op = ContourOperations::VariableWidthStroke { data: new_data };

                v.begin_layer_modification("VWS dialog modification.");
                v.with_active_layer_mut(|layer| layer.outline[contour_idx].operation = Some(new_op.clone()) );
                v.end_layer_modification();
            }
        }
    }

    pub fn build_vws_settings_window(&self, v: &mut Editor, i: &Interface, ui: &mut imgui::Ui) {
        let (tx, ty, tw, th) = i.get_tools_dialog_rect();

        // if we don't have a contour selected we don't draw this
        if v.contour_idx.is_none() {
            return;
        }

        let _contour_idx = v.contour_idx.unwrap();

        imgui::Window::new(imgui::im_str!("VWS Settings"))
            .bg_alpha(1.) // See comment on fn redraw_skia
            .flags(
                imgui::WindowFlags::NO_RESIZE
                    | imgui::WindowFlags::NO_MOVE
                    | imgui::WindowFlags::NO_COLLAPSE,
            )
            .position(
                [tx, ty],
                imgui::Condition::Always,
            )
            .size(
                [tw, th],
                imgui::Condition::Always,
            )
            .build(ui, || {
                self.build_and_check_vws_cap_combo(v, ui);
                self.build_and_check_vws_join_combo(v, ui);
            });
    }
}
