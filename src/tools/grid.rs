use crate::user_interface::grid::Grid;
use crate::tool_behaviors::pan::PanBehavior;
use crate::user_interface::util::imgui_decimal_text_field;
use super::prelude::*;

#[derive(Clone)]
pub struct GridTool {
}

impl Tool for GridTool {
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent { event_type, mouse_info} => {
                match event_type {
                    MouseEventType::Pressed => { v.set_behavior(Box::new(PanBehavior::new(i.viewport.clone(), mouse_info))); }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn ui(&mut self, _v: &mut Editor, i: &mut Interface, ui: &mut Ui) {
        self.grid_settings(i, ui);
    }
}
 
impl GridTool {
    pub fn new() -> Self {
        Self { }
    }

    pub fn grid_settings(&mut self, i: &mut Interface, ui: &imgui::Ui) {
        let (tx, ty, tw, th) = i.get_tools_dialog_rect();

        imgui::Window::new(
                &imgui::ImString::new("Grid")
            )
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
                let old_active = i.grid.is_some();
                let mut active = old_active;

                ui.checkbox(imgui::im_str!("Active"), &mut active);

                if !active {
                    i.grid = None;
                } else if !old_active && active { 
                    i.grid = Some(Grid {
                        offset: 0.,
                        spacing: 30.,
                        slope: None,
                    })
                }

                if let Some(grid) = &mut i.grid {
                    imgui_decimal_text_field("Spacing", ui, &mut grid.spacing);
                    imgui_decimal_text_field("Offset", ui, &mut grid.offset);

                    let old_italic = grid.slope.is_some();
                    let mut italic = grid.slope.is_some();
                    ui.checkbox(imgui::im_str!("Italic"), &mut italic);
                    if italic != old_italic && italic {
                        grid.slope = Some(0.5);
                    } else if italic != old_italic && !italic {
                        grid.slope = None;
                    }

                    if let Some(slope) = grid.slope {
                        let mut old_slope = slope.clone();
                        imgui_decimal_text_field("Slope", ui, &mut old_slope);
                        grid.slope = Some(old_slope);
                    }

                    grid.offset %= grid.spacing;
                }
            });
    }
}
