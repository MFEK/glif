use crate::user_interface::grid::Grid;

use super::prelude::*;

#[derive(Clone)]
pub struct GridTool {
    last_position: Option<(f32, f32)>,
}


impl Tool for GridTool {
    fn handle_event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::Ui { ui } => {
                self.grid_settings(v, i, ui);
            }
            _ => {}
        }
    }
}

fn imgui_decimal_text_field(label: &str, ui: &imgui::Ui, data: &mut f32) {
    let mut x = imgui::im_str!("{}", data);
    let label = imgui::ImString::new(label);
    let entered;
    {
    let it = ui.input_text(&label, &mut x);
    entered = it.enter_returns_true(true)
        .chars_decimal(true)
        .chars_noblank(true)
        .auto_select_all(true)
        .build();
    }
    if entered {
        if x.to_str().len() > 0 {
            let new_x: f32 = x.to_str().parse().unwrap();
            *data = new_x;
        }
    }
}
 
impl GridTool {
    pub fn new() -> Self {
        Self { last_position: None }
    }

    pub fn grid_settings(&mut self, v: &mut Editor, i: &mut Interface, ui: &imgui::Ui) {
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
                    })
                }

                if let Some(grid) = &mut i.grid {
                    imgui_decimal_text_field("Spacing", ui, &mut grid.spacing);
                    imgui_decimal_text_field("Offset", ui, &mut grid.offset);
                }
            });
    }
}
