use super::prelude::*;
use crate::io::{load_glif, save_glif};
use crate::state::Follow;
use glifparser::{Handle, WhichHandle};
use sdl2::keyboard::Mod;
use skulpin::skia_safe::{Canvas, Paint, PaintStyle, Path as SkiaPath};
use MFEKMath::variable_width_stroking::{generate_vws_lib, InterpolationType};
use MFEKMath::{
    parse_vws_lib, variable_width_stroke, CapType, Evaluate, JoinType, Piecewise, VWSContour,
    VWSHandle, VWSSettings, Vector,
};

use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process;

use imgui;

//
// IPC
//
pub fn export_vws<F: AsRef<OsStr> + AsRef<Path> + Clone>(filename: F) {
    let qmdbin = mfek_ipc::module_name("MFEKstroke".into());

    let cur_file = STATE.with(|v| {
        save_glif(v);
        v.borrow().glyph.as_ref().unwrap().filename.clone()
    });

    let command = process::Command::new(qmdbin)
        .arg("VWS")
        .arg("-i")
        .arg(cur_file.clone())
        .arg("-o")
        .arg(filename.clone())
        .output();

    match command {
        Ok(output) => {
            println!("{:?}", output);

            if !output.status.success() {
                return;
            }

            // this step makes ../glyphs/E.glif and E.glif equal
            let canonical_cur_file = fs::canonicalize(&cur_file);
            let canonical_filename = fs::canonicalize(&filename);
            // we've got to clear the VWS contours after this or we're gonna crash
            if canonical_cur_file.is_ok()
                && canonical_filename.is_ok()
                && canonical_cur_file.unwrap() == canonical_filename.unwrap()
            {
                STATE.with(|v| {
                    v.borrow_mut().vws_contours = Vec::new();
                });
            }

            load_glif(cur_file.clone());
        }
        Err(output) => println!("{:?}", output),
    }
}

//
// UI
//
fn build_and_check_vws_cap_combo(ui: &imgui::Ui) {
    let contour_idx = TOOL_DATA.with(|v| v.borrow().contour.unwrap());

    let _vws_contour = STATE.with(|v| get_vws_contour(v, contour_idx));

    if let Some(mut vws_contour) = _vws_contour {
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
            vws_contour.cap_start_type = s_selection;
            vws_contour.cap_end_type = e_selection;
            STATE.with(|v| {
                set_vws_contour_by_value(v, contour_idx, vws_contour);
                generate_previews(v);
            });
        }
    }
}

fn build_and_check_vws_join_combo(ui: &imgui::Ui) {
    let contour_idx = TOOL_DATA.with(|v| v.borrow().contour.unwrap());

    let _vws_contour = STATE.with(|v| get_vws_contour(v, contour_idx));

    if let Some(mut vws_contour) = _vws_contour {
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

        vws_contour.join_type = new_selection;
        STATE.with(|v| {
            set_vws_contour_by_value(v, contour_idx, vws_contour);
            generate_previews(v);
        });
    }
}

fn join_type_to_idx(jt: JoinType) -> usize {
    match jt {
        JoinType::Round => 0,
        JoinType::Miter => 1,
        JoinType::Bevel => 2,
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

// we redefine these constants here for positioning our window
const TOOLBOX_OFFSET_X: f32 = 10.;
const TOOLBOX_OFFSET_Y: f32 = TOOLBOX_OFFSET_X;
const TOOLBOX_WIDTH: f32 = 55.;
const TOOLBOX_HEIGHT: f32 = 220.;

pub fn build_vws_settings_window(ui: &mut imgui::Ui) {
    let countour_idx = TOOL_DATA.with(|v| v.borrow().contour);

    // if we don't have a contour selected we don't draw this
    if countour_idx.is_none() {
        return;
    }

    imgui::Window::new(imgui::im_str!("VWS Settings"))
        .bg_alpha(1.) // See comment on fn redraw_skia
        .flags(
            #[rustfmt::skip]
              imgui::WindowFlags::NO_RESIZE
                | imgui::WindowFlags::NO_MOVE
                | imgui::WindowFlags::NO_COLLAPSE,
        )
        .position(
            [TOOLBOX_OFFSET_X, TOOLBOX_OFFSET_Y + TOOLBOX_HEIGHT + 30.],
            imgui::Condition::Always,
        )
        .size(
            [TOOLBOX_WIDTH * 3., TOOLBOX_HEIGHT / 2.],
            imgui::Condition::Always,
        )
        .build(ui, || {
            build_and_check_vws_cap_combo(ui);
            ui.separator();
            build_and_check_vws_join_combo(ui);
        });
}

//
// Loading
//

pub fn on_load_glif() {
    STATE.with(|v| {
        let mut _v = v.borrow_mut();

        if let Some(vws_contours) = parse_vws_lib(&_v.glyph.as_ref().unwrap().glif) {
            println!("herp");
            _v.vws_contours = vws_contours.0;
            _v.glyph.as_mut().unwrap().glif.lib = Some(vws_contours.1);
        }
    });

    STATE.with(|v| generate_previews(v))
}

pub fn generate_lib(vwscontours: Vec<VWSContour>) -> Option<xmltree::Element> {
    return generate_vws_lib(&vwscontours);
}

//
// Manipulating
//
fn get_vws_contour(
    v: &RefCell<state::State<Option<state::PointData>>>,
    contour_idx: usize,
) -> Option<VWSContour> {
    for vwscontour in v.borrow().vws_contours.iter() {
        if vwscontour.id == contour_idx {
            return Some(vwscontour.clone());
        }
    }

    None
}

fn set_vws_contour_by_value(
    v: &RefCell<state::State<Option<state::PointData>>>,
    contour_idx: usize,
    vws_contour: VWSContour,
) {
    let mut _v = v.borrow_mut();
    let mut to_remove = None;

    for (idx, vwscontour) in _v.vws_contours.iter().enumerate() {
        if vwscontour.id == contour_idx {
            to_remove = Some(idx);
        }
    }

    if let Some(to_remove) = to_remove {
        _v.vws_contours.remove(to_remove);
    }

    _v.vws_contours.push(vws_contour);
}

fn get_vws_contour_idx(
    v: &RefCell<state::State<Option<state::PointData>>>,
    contour_idx: usize,
) -> Option<usize> {
    for (idx, vwscontour) in v.borrow().vws_contours.iter().enumerate() {
        if vwscontour.id == contour_idx {
            return Some(idx);
        }
    }

    None
}

fn fix_vws_contour(v: &RefCell<state::State<Option<state::PointData>>>, contour_idx: usize) {
    let contour_size = get_outline!(v)[contour_idx].len();
    let vws_contour_size = v.borrow().vws_contours[contour_idx].handles.len();

    let difference = vws_contour_size - (contour_size + 1);
    if difference != 0 {
        for _i in 0..difference {
            v.borrow_mut().vws_contours[contour_idx]
                .handles
                .push(VWSHandle {
                    left_offset: 10.,
                    right_offset: 10.,
                    interpolation: InterpolationType::Linear,
                    tangent_offset: 0.,
                })
        }
    }
}

fn generate_vws_contour(v: &RefCell<state::State<Option<state::PointData>>>, contour_idx: usize) {
    let mut new_vws_contour = VWSContour {
        handles: Vec::new(),
        id: contour_idx,
        cap_start_type: CapType::Round,
        cap_end_type: CapType::Round,
        join_type: JoinType::Round,
    };

    for _i in 0..get_outline!(v)[contour_idx].len() + 1 {
        new_vws_contour.handles.push(VWSHandle {
            left_offset: 10.,
            right_offset: 10.,
            interpolation: InterpolationType::Linear,
            tangent_offset: 0.,
        })
    }

    v.borrow_mut().vws_contours.push(new_vws_contour);
}

fn get_vws_handle(
    v: &RefCell<state::State<Option<state::PointData>>>,
    vcontour: Option<usize>,
    handle_idx: usize,
) -> VWSHandle {
    if let Some(vc) = vcontour {
        // if the contour exists but this handle doesn't we're gonna add handles until we've got
        // the right amount
        if v.borrow().vws_contours[vc].handles.len() < handle_idx {
            fix_vws_contour(v, vc);
        }

        return v.borrow().vws_contours[vc].handles[handle_idx].clone();
    }

    return VWSHandle {
        left_offset: 10.,
        right_offset: 10.,
        interpolation: InterpolationType::Linear,
        tangent_offset: 0.,
    };
}

fn set_vws_handle(
    v: &RefCell<state::State<Option<state::PointData>>>,
    contour_idx: usize,
    handle_idx: usize,
    side: WhichHandle,
    pos: f64,
    tangent: f64,
) {
    if get_vws_contour_idx(v, contour_idx).is_none() {
        generate_vws_contour(v, contour_idx);
    }

    // we know this contour exists now
    let vws_contour = get_vws_contour_idx(v, contour_idx).unwrap();

    let id = v.borrow().vws_contours[vws_contour].id;
    let contour_pw = Piecewise::from(&get_outline!(v)[id]);

    let side_multiplier = match side {
        WhichHandle::A => 1.,
        WhichHandle::B => -1.,
        _ => unreachable!(),
    };

    let mirror = TOOL_DATA.with(|p| p.borrow().follow == Follow::Mirror);
    let constrain = TOOL_DATA.with(|p| p.borrow().ctrl == true);

    let tangent_offset = if constrain {
        0.
    } else {
        side_multiplier * tangent
    };

    if handle_idx == 0 && contour_pw.is_closed() {
        let last_handle = v.borrow().vws_contours[vws_contour].handles.len() - 1;

        v.borrow_mut().vws_contours[vws_contour].handles[last_handle].tangent_offset =
            tangent_offset;

        match side {
            WhichHandle::A => {
                v.borrow_mut().vws_contours[vws_contour].handles[last_handle].left_offset = pos
            }
            WhichHandle::B => {
                v.borrow_mut().vws_contours[vws_contour].handles[last_handle].right_offset = pos
            }
            _ => {}
        }

        if mirror {
            v.borrow_mut().vws_contours[vws_contour].handles[last_handle].left_offset = pos;
            v.borrow_mut().vws_contours[vws_contour].handles[last_handle].right_offset = pos;
            v.borrow_mut().vws_contours[vws_contour].handles[last_handle].tangent_offset = tangent;
        } else {
            v.borrow_mut().vws_contours[vws_contour].handles[last_handle].tangent_offset =
                tangent_offset;
        }
    }

    match side {
        WhichHandle::A => {
            v.borrow_mut().vws_contours[vws_contour].handles[handle_idx].left_offset = pos
        }
        WhichHandle::B => {
            v.borrow_mut().vws_contours[vws_contour].handles[handle_idx].right_offset = pos
        }
        _ => {
            v.borrow_mut().vws_contours[vws_contour].handles[handle_idx].left_offset = pos;
            v.borrow_mut().vws_contours[vws_contour].handles[handle_idx].right_offset = pos;
        }
    }

    if mirror {
        v.borrow_mut().vws_contours[vws_contour].handles[handle_idx].left_offset = pos;
        v.borrow_mut().vws_contours[vws_contour].handles[handle_idx].right_offset = pos;
        v.borrow_mut().vws_contours[vws_contour].handles[handle_idx].tangent_offset =
            tangent_offset;
    } else {
        v.borrow_mut().vws_contours[vws_contour].handles[handle_idx].tangent_offset =
            tangent_offset;
    }
}

fn set_all_vws_handles(
    v: &RefCell<state::State<Option<state::PointData>>>,
    contour_idx: usize,
    side: WhichHandle,
    pos: f64,
) {
    if get_vws_contour_idx(v, contour_idx).is_none() {
        generate_vws_contour(v, contour_idx);
    }

    STATE.with(|v| {
        // we know this contour exists now
        let vws_contour = get_vws_contour_idx(v, contour_idx).unwrap();

        let mut borrowed_v = v.borrow_mut();
        for handle_idx in 0..borrowed_v.vws_contours[vws_contour].handles.len() {
            let follow = TOOL_DATA.with(|v| v.borrow().follow);

            if follow == Follow::Mirror {
                borrowed_v.vws_contours[vws_contour].handles[handle_idx].left_offset = pos;
                borrowed_v.vws_contours[vws_contour].handles[handle_idx].right_offset = pos;
            } else {
                match side {
                    WhichHandle::A => {
                        borrowed_v.vws_contours[vws_contour].handles[handle_idx].left_offset = pos
                    }
                    WhichHandle::B => {
                        borrowed_v.vws_contours[vws_contour].handles[handle_idx].right_offset = pos
                    }
                    _ => {} // should be unreachable
                }
            }
        }
    });
}

fn generate_previews(v: &RefCell<state::State<Option<state::PointData>>>) {
    let mut new_previews = Vec::new();

    for vws_contour in &v.borrow().vws_contours {
        let contour_pw = Piecewise::from(&get_outline!(v)[vws_contour.id]);

        let settings = VWSSettings {
            cap_custom_start: None,
            cap_custom_end: None,
        };

        let vws_output = variable_width_stroke(&contour_pw, &vws_contour, &settings);

        for contour in vws_output.segs {
            new_previews.push(contour.to_contour());
        }
    }

    v.borrow_mut().vws_previews = Some(new_previews);
}

fn mouse_coords_to_handle_space(
    v: &RefCell<state::State<Option<state::PointData>>>,
    contour_idx: usize,
    handle_idx: usize,
    side: WhichHandle,
    mouse_pos: Vector,
) -> (f64, f64) {
    let (start_pos, tangent, _handle_pos) = get_vws_handle_pos(v, contour_idx, handle_idx, side);
    let side_multiplier = match side {
        WhichHandle::A => -1.,
        WhichHandle::B => 1.,
        _ => unreachable!(),
    };

    let tangent = tangent.normalize();
    let normal = Vector {
        x: tangent.y,
        y: -tangent.x,
    }
    .normalize();
    let mouse_vec = start_pos - mouse_pos;
    let mouse_vec_normal = mouse_vec.normalize();

    println!(
        "tangent: {:?} final: {:?}",
        tangent,
        mouse_vec_normal.dot(tangent) * mouse_vec.magnitude()
    );

    //return mouse_vec_normal.dot(handle_vec) * mouse_vec.magnitude();
    let normal_offset = f64::max(
        mouse_vec_normal.dot(normal) * mouse_vec.magnitude() * side_multiplier,
        0.,
    );
    let tangent_offset = mouse_vec_normal.dot(tangent) * mouse_vec.magnitude();

    (normal_offset, tangent_offset)
}

fn get_vws_handle_pos(
    v: &RefCell<state::State<Option<state::PointData>>>,
    contour_idx: usize,
    handle_idx: usize,
    side: WhichHandle,
) -> (Vector, Vector, Vector) {
    let vws_contour = get_vws_contour_idx(v, contour_idx);
    let contour_pw = Piecewise::from(&get_outline!(v)[contour_idx]);

    if handle_idx < contour_pw.segs.len() {
        let vws_handle = get_vws_handle(v, vws_contour, handle_idx);
        let bezier = &contour_pw.segs[handle_idx];
        let start_point = bezier.start_point();
        let tangent = bezier.tangent_at(0.).normalize();
        let normal = Vector {
            x: tangent.y,
            y: -tangent.x,
        }
        .normalize();

        let max_tangent = f64::max(vws_handle.right_offset, vws_handle.left_offset);

        let scaled_tangent_offset = match side {
            WhichHandle::A => vws_handle.left_offset / max_tangent,
            WhichHandle::B => vws_handle.right_offset / max_tangent,
            WhichHandle::Neither => panic!("Should be unreachable!"),
        };

        match side {
            WhichHandle::A => {
                return (
                    start_point,
                    tangent,
                    start_point
                        + normal * vws_handle.left_offset
                        + tangent * -vws_handle.tangent_offset * scaled_tangent_offset,
                )
            }
            WhichHandle::B => {
                return (
                    start_point,
                    tangent,
                    start_point
                        + normal * -vws_handle.right_offset
                        + tangent * vws_handle.tangent_offset * scaled_tangent_offset,
                )
            }
            _ => panic!("Should be unreachable!"),
        }
    } else {
        let vws_handle = get_vws_handle(v, vws_contour, handle_idx);
        let bezier = &contour_pw.segs.last().unwrap();
        let start_point = bezier.end_point();
        let tangent = bezier.tangent_at(1.).normalize();
        let normal = Vector {
            x: tangent.y,
            y: -tangent.x,
        }
        .normalize();

        match side {
            WhichHandle::A => {
                return (
                    start_point,
                    tangent,
                    start_point
                        + normal * vws_handle.left_offset
                        + tangent * -vws_handle.tangent_offset,
                )
            }
            WhichHandle::B => {
                return (
                    start_point,
                    tangent,
                    start_point
                        + normal * -vws_handle.right_offset
                        + tangent * vws_handle.tangent_offset,
                )
            }
            _ => panic!("Should be unreachable!"),
        }
    }
}

fn vws_clicked_point_or_handle(
    position: (f64, f64),
    v: &RefCell<state::State<Option<state::PointData>>>,
) -> Option<(usize, usize, WhichHandle)> {
    let factor = v.borrow().factor;
    let mposition = update_mousepos(position, &v, true);
    let _contour_idx = 0;
    let _point_idx = 0;

    for (contour_idx, contour) in get_outline!(v).iter().enumerate() {
        let contour_pw = Piecewise::from(contour);

        let size = ((POINT_RADIUS * 2.) + (POINT_STROKE_THICKNESS * 2.)) * (1. / factor);
        for vws_handle_idx in 0..contour_pw.segs.len() {
            let handle_pos_left =
                get_vws_handle_pos(v, contour_idx, vws_handle_idx, WhichHandle::A).2;
            let handle_pos_right =
                get_vws_handle_pos(v, contour_idx, vws_handle_idx, WhichHandle::B).2;

            let handle_left_point = SkPoint::new(
                calc_x(handle_pos_left.x as f32) - (size / 2.),
                calc_y(handle_pos_left.y as f32) - (size / 2.),
            );
            let handle_left_rect = SkRect::from_point_and_size(handle_left_point, (size, size));

            let handle_right_point = SkPoint::new(
                calc_x(handle_pos_right.x as f32) - (size / 2.),
                calc_y(handle_pos_right.y as f32) - (size / 2.),
            );
            let handle_right_rect = SkRect::from_point_and_size(handle_right_point, (size, size));

            let sk_mpos = SkPoint::new(mposition.0 as f32, mposition.1 as f32);

            if handle_left_rect.contains(sk_mpos) {
                return Some((contour_idx, vws_handle_idx, WhichHandle::A));
            } else if handle_right_rect.contains(sk_mpos) {
                return Some((contour_idx, vws_handle_idx, WhichHandle::B));
            }
        }

        if contour.first().unwrap().ptype == glifparser::PointType::Move {
            let vws_handle_idx = contour_pw.segs.len();

            let handle_pos_left =
                get_vws_handle_pos(v, contour_idx, vws_handle_idx, WhichHandle::A).2;
            let handle_pos_right =
                get_vws_handle_pos(v, contour_idx, vws_handle_idx, WhichHandle::B).2;

            let handle_left_point = SkPoint::new(
                calc_x(handle_pos_left.x as f32) - (size / 2.),
                calc_y(handle_pos_left.y as f32) - (size / 2.),
            );
            let handle_left_rect = SkRect::from_point_and_size(handle_left_point, (size, size));

            let handle_right_point = SkPoint::new(
                calc_x(handle_pos_right.x as f32) - (size / 2.),
                calc_y(handle_pos_right.y as f32) - (size / 2.),
            );
            let handle_right_rect = SkRect::from_point_and_size(handle_right_point, (size, size));

            let sk_mpos = SkPoint::new(mposition.0 as f32, mposition.1 as f32);

            if handle_left_rect.contains(sk_mpos) {
                return Some((contour_idx, vws_handle_idx, WhichHandle::A));
            } else if handle_right_rect.contains(sk_mpos) {
                return Some((contour_idx, vws_handle_idx, WhichHandle::B));
            }
        }
    }

    None
}

pub fn mouse_pressed(
    position: (f64, f64),
    v: &RefCell<state::State<Option<state::PointData>>>,
    meta: MouseMeta,
) -> bool {
    match vws_clicked_point_or_handle(position, v) {
        Some((ci, pi, wh)) => TOOL_DATA.with(|p| {
            let follow: Follow = meta.into();
            debug!(
                "Clicked point: {:?} {:?}. Follow behavior: {}",
                get_outline!(v)[ci][pi],
                wh,
                follow
            );
            p.borrow_mut().contour = Some(ci);
            p.borrow_mut().cur_point = Some(pi);
            p.borrow_mut().follow = follow;
            p.borrow_mut().handle = wh;
            p.borrow_mut().shift = meta.modifiers.shift;
            p.borrow_mut().ctrl = meta.modifiers.ctrl;

            true
        }),
        None => false,
    };

    false
}

// Placeholder
pub fn mouse_button<T>(
    _position: (f64, f64),
    _v: &RefCell<state::State<T>>,
    _meta: MouseMeta,
) -> bool {
    false
}

pub fn mouse_released(
    _position: (f64, f64),
    v: &RefCell<state::State<Option<state::PointData>>>,
    _meta: MouseMeta,
) -> bool {
    TOOL_DATA.with(|p| {
        p.borrow_mut().cur_point = None;
        true
    })
}

/// Get indexes stored by clicked_point_or_handle and move the points they refer to around.
pub fn mouse_moved(
    position: (f64, f64),
    v: &RefCell<state::State<Option<state::PointData>>>,
) -> bool {
    let mposition = update_mousepos(position, &v, false);
    if !v.borrow().mousedown {
        return false;
    }

    let x = calc_x(mposition.0 as f32);
    let y = calc_y(mposition.1 as f32);
    let contour = TOOL_DATA.with(|p| p.borrow().contour);
    let cur_point = TOOL_DATA.with(|p| p.borrow().cur_point);
    let which_handle = TOOL_DATA.with(|p| p.borrow().handle);
    let shift = TOOL_DATA.with(|p| p.borrow().shift);

    match (contour, cur_point, which_handle) {
        // A control point (A or B) is being moved.
        (Some(ci), Some(pi), wh) => {
            println!("{:?}", wh);
            let (normal_offset, tangent_offset) = mouse_coords_to_handle_space(
                v,
                ci,
                pi,
                wh,
                Vector {
                    x: x as f64,
                    y: y as f64,
                },
            );
            // if shift is held down we scale all the points
            if shift {
                set_all_vws_handles(v, ci, wh, normal_offset);
            } else {
                set_vws_handle(v, ci, pi, wh, normal_offset, tangent_offset);
            }

            generate_previews(v);
            false
        }
        _ => false,
    };

    true
}

pub fn update_previews(
    position: (f64, f64),
    v: &RefCell<state::State<Option<state::PointData>>>,
) -> bool {
    let mposition = update_mousepos(position, &v, false);
    if !v.borrow().mousedown {
        return false;
    }
    generate_previews(v);

    true
}

pub fn should_draw_contour(
    v: &RefCell<state::State<Option<state::PointData>>>,
    idx: usize,
) -> bool {
    if get_vws_contour_idx(v, idx).is_some() {
        return false;
    }

    return true;
}

pub fn draw_handles(canvas: &mut Canvas) {
    STATE.with(|v| {
        let factor = v.borrow().factor;

        for (contour_idx, contour) in get_outline!(v).iter().enumerate() {
            let contour_pw = Piecewise::from(contour);

            for (vws_handle_idx, bezier) in contour_pw.segs.iter().enumerate() {
                let start_point = bezier.start_point();
                let handle_pos_left =
                    get_vws_handle_pos(v, contour_idx, vws_handle_idx, WhichHandle::A).2;
                let handle_pos_right =
                    get_vws_handle_pos(v, contour_idx, vws_handle_idx, WhichHandle::B).2;

                let mut path = SkiaPath::new();
                let mut paint = Paint::default();

                paint.set_anti_alias(true);
                paint.set_color(RIB_STROKE);
                paint.set_stroke_width(HANDLEBAR_THICKNESS * (1. / factor));
                paint.set_style(PaintStyle::Stroke);

                path.move_to((
                    calc_x(handle_pos_left.x as f32),
                    calc_y(handle_pos_left.y as f32),
                ));
                path.line_to((calc_x(start_point.x as f32), calc_y(start_point.y as f32)));
                path.line_to((
                    calc_x(handle_pos_right.x as f32),
                    calc_y(handle_pos_right.y as f32),
                ));

                canvas.draw_path(&path, &paint);
            }

            if contour.first().unwrap().ptype == glifparser::PointType::Move {
                let vws_handle_idx = contour_pw.segs.len();
                let bezier = contour_pw.segs.last().unwrap();
                let start_point = bezier.end_point();

                let handle_pos_left =
                    get_vws_handle_pos(v, contour_idx, vws_handle_idx, WhichHandle::A).2;
                let handle_pos_right =
                    get_vws_handle_pos(v, contour_idx, vws_handle_idx, WhichHandle::B).2;

                let mut path = SkiaPath::new();
                let mut paint = Paint::default();

                paint.set_anti_alias(true);
                paint.set_color(RIB_STROKE);
                paint.set_stroke_width(HANDLEBAR_THICKNESS * (1. / factor));
                paint.set_style(PaintStyle::Stroke);

                path.move_to((
                    calc_x(handle_pos_left.x as f32),
                    calc_y(handle_pos_left.y as f32),
                ));
                path.line_to((calc_x(start_point.x as f32), calc_y(start_point.y as f32)));
                path.line_to((
                    calc_x(handle_pos_right.x as f32),
                    calc_y(handle_pos_right.y as f32),
                ));

                canvas.draw_path(&path, &paint);
            }
        }
    })
}
