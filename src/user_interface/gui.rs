use std::{cell::RefCell, rc::Rc};

use crate::{editor::Editor, tools::{EditorEvent, ToolEnum}, user_interface::InputPrompt};

use super::{Interface, icons};

// These are before transformation by STATE.dpi (glutin scale_factor)
pub const TOOLBOX_OFFSET_X: f32 = 10.;
pub const TOOLBOX_OFFSET_Y: f32 = TOOLBOX_OFFSET_X;
pub const TOOLBOX_WIDTH: f32 = 52.;

use enum_unitary::Bounded as _;
use lazy_static::lazy_static;
lazy_static! {
    pub static ref NUM_TOOLS: usize = ToolEnum::max_value() as usize;
    pub static ref TOOLBOX_HEIGHT: f32 = (*NUM_TOOLS * 20 + (*NUM_TOOLS * 10)) as f32;
}

pub const LAYERBOX_WIDTH: f32 = 250.;
pub const LAYERBOX_HEIGHT: f32 = 250.;

use glifparser::glif::LayerOperation;
use imgui::{self, ColorStackToken, Context, DrawData, FontId, Key, StyleColor, StyleVar};
use imgui_sdl2::ImguiSdl2;
use imgui_skia_renderer::Renderer;
use sdl2::{event::Event, mouse::MouseState, video::Window};

pub struct ImguiManager {
    pub imgui_context: Context,
    pub imgui_renderer: imgui_skia_renderer::Renderer,
    pub imgui_sdl2: imgui_sdl2::ImguiSdl2,
}

impl ImguiManager {
    pub fn new(window: &Window) -> Self {
        let mut imgui = Context::create();
        {
            // Fix incorrect colors with sRGB framebuffer
            fn imgui_gamma_to_linear(col: [f32; 4]) -> [f32; 4] {
                let x = col[0].powf(2.2);
                let y = col[1].powf(2.2);
                let z = col[2].powf(2.2);
                let w = 1.0 - (1.0 - col[3]).powf(2.2);
                [x, y, z, w]
            }
    
            let style = imgui.style_mut();
            for col in 0..style.colors.len() {
                style.colors[col] = imgui_gamma_to_linear(style.colors[col]);
            }
        }
    
        imgui.set_ini_filename(None);
        imgui.style_mut().use_light_colors();
    
        // TODO: Implement proper DPI scaling
        let scale_factor = 1.;
        let font_size = (16.0 * scale_factor) as f32;
        let icon_font_size = (36.0 * scale_factor) as f32;
    
        static ICON_FONT_TTF_DATA: &[u8] = include_bytes!("../../resources/fonts/icons.ttf");
    
        let id = imgui.fonts().add_font(&[
            imgui::FontSource::TtfData {
                data: &crate::system_fonts::SYSTEMSANS.data,
                size_pixels: font_size,
                config: Some(imgui::FontConfig {
                    oversample_h: 3,
                    oversample_v: 3,
                    glyph_ranges: imgui::FontGlyphRanges::from_slice(&[
                        0x0020 as u16,
                        0x00FF as u16,
                        0x03B8 as u16, // for Greek theta
                        0x03B8 as u16, // for Greek theta
                        0,
                    ]),
                    ..Default::default()
                }),
            },
            imgui::FontSource::TtfData {
                data: ICON_FONT_TTF_DATA,
                size_pixels: icon_font_size,
                config: Some(imgui::FontConfig {
                    glyph_ranges: imgui::FontGlyphRanges::from_slice(&[
                        0xF000 as u16,
                        0xF100 as u16,
                        0,
                    ]),
                    ..Default::default()
                }),
            },
        ]);
    
        let id1= imgui.fonts().add_font(&[
            imgui::FontSource::TtfData {
                data: &crate::system_fonts::SYSTEMSANS.data,
                size_pixels: font_size,
                config: Some(imgui::FontConfig {
                    oversample_h: 3,
                    oversample_v: 3,
                    ..Default::default()
                }),
            },
            imgui::FontSource::TtfData {
                data: ICON_FONT_TTF_DATA,
                size_pixels: 28.,
                config: Some(imgui::FontConfig {
                    glyph_ranges: imgui::FontGlyphRanges::from_slice(&[
                        0xF000 as u16,
                        0xF100 as u16,
                        0,
                    ]),
                    ..Default::default()
                }),
            },
        ]);
    
        FONT_IDS.with(|ids| {
            ids.borrow_mut().push(id);
            ids.borrow_mut().push(id1);
        });
        PROMPT_STR.with(|prompt_str| {
            prompt_str.borrow_mut().reserve(256)
        });
        
        let imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);
        let imgui_renderer = Renderer::new(&mut imgui);
        return ImguiManager {
            imgui_context: imgui,
            imgui_renderer: imgui_renderer,
            imgui_sdl2: imgui_sdl2
        }
    }
    
    pub fn build_and_check_button(v: &mut Editor, ui: &imgui::Ui, mode: ToolEnum, icon: &[u8]) {
        let mut pop_me = None;
        if v.get_tool() == mode {
            pop_me = Some(ui.push_style_color(imgui::StyleColor::Button, [0., 0., 0., 0.2]));
        }
        // Icons are always constant so this is not really unsafe.
        ui.button(
            unsafe { imgui::ImStr::from_utf8_with_nul_unchecked(icon) },
            [0., 30.],
        );
        if ui.is_item_clicked(imgui::MouseButton::Left) {
            v.set_tool(mode);
        }
        if let Some(p) = pop_me {
            p.pop(ui);
        }
    }
    
    pub fn build_and_check_layer_list(v: &mut Editor, i: &mut Interface, ui: &imgui::Ui) {
    
        let active_layer = v.get_active_layer();
    
        let pop_me = ui.push_style_color(imgui::StyleColor::Button, [0., 0., 0., 0.2]);
    
        ui.button(unsafe { imgui::ImStr::from_utf8_with_nul_unchecked(icons::PLUS) }, [0., 0.]);
        //ui.push_item_width(-0.5);
        if ui.is_item_clicked(imgui::MouseButton::Left) {
            v.new_layer();
        }
    
        ui.same_line(0.);
        ui.button(unsafe { imgui::ImStr::from_utf8_with_nul_unchecked(icons::MINUS) }, [0., 0.]);
        ui.push_item_width(-0.5);
        if ui.is_item_clicked(imgui::MouseButton::Left) {
            v.delete_layer(active_layer, true);
        }
    
        ui.same_line(0.);
        ui.button(unsafe { imgui::ImStr::from_utf8_with_nul_unchecked(icons::ARROWUP) }, [0., 0.]);
        if ui.is_item_clicked(imgui::MouseButton::Left) {
            if active_layer != 0 {
                let _start_layer_type = v.with_glyph(|glif| glif.layers[active_layer].operation.clone());
                let _target_layer_type = v.with_glyph(|glif| glif.layers[active_layer-1].operation.clone());
    
                v.swap_layers(active_layer, active_layer-1, true);
            }
        }
    
        let layer_count = v.get_layer_count();
        ui.same_line(0.);
        ui.button(unsafe { imgui::ImStr::from_utf8_with_nul_unchecked(icons::ARROWDOWN) }, [0., 0.]);
        if ui.is_item_clicked(imgui::MouseButton::Left) {
            if active_layer != layer_count-1 {
                v.swap_layers(active_layer, active_layer+1, true);
            }
        }
    
        pop_me.pop(ui);
        
        ui.separator();
    
        for layer in 0 .. layer_count {
            let layer_op = v.with_glyph(|glif| glif.layers[layer].operation.clone());
            let layer_temp_name = imgui::im_str!("{0}", v.with_glyph(|glif| { glif.layers[layer].name.clone() }));
            let im_str = imgui::ImString::from(layer_temp_name);
    
            let font_token = ui.push_font(FONT_IDS.with(|ids| { ids.borrow()[1] }));
            let no_padding = ui.push_style_var(StyleVar::ItemSpacing([0., 0.]));
            let custom_button_color = ui.push_style_color(imgui::StyleColor::Button, ui.style_color(StyleColor::WindowBg));
    
            if layer_op.is_some() {
                ui.dummy([28., 0.]);
                ui.same_line(0.);
            }
            let layer_visible = v.with_glyph(|glif| glif.layers[layer].visible);
            let eye_con = if layer_visible { icons::OPENEYE } else { icons::CLOSEDEYE };
            ui.button(unsafe { imgui::ImStr::from_utf8_with_nul_unchecked(eye_con) }, [0., 0.]);
            if ui.is_item_clicked(imgui::MouseButton::Left) {
                let active_layer = v.get_active_layer();
                v.set_active_layer(layer);
    
                v.begin_layer_modification("Toggled layer visibility.");
                v.with_active_layer_mut(|layer| layer.visible = !layer.visible );
                v.end_layer_modification();
    
                v.set_active_layer(active_layer);
            }
    
            ui.same_line(0.);
            ui.button(unsafe { imgui::ImStr::from_utf8_with_nul_unchecked(icons::RENAME) }, [0., 0.]);
            if ui.is_item_clicked(imgui::MouseButton::Left) {
                i.push_prompt(InputPrompt::Text {
                    label: "Layer name:".to_string(),
                    default: v.with_glyph(|glif| glif.layers[layer].name.clone()),
                    func: Rc::new(move |editor, string| {
                        let active_layer = editor.get_active_layer();
                        editor.set_active_layer(layer);
        
                        editor.begin_layer_modification("Renamed layer.");
                        editor.with_active_layer_mut(|layer| layer.name = string.clone());
                        editor.end_layer_modification();
        
                        editor.set_active_layer(active_layer);
                    }),
                });
            }
            ui.same_line(0.);
            
            let current_operation = v.with_glyph(|glif| glif.layers[layer].operation.clone() );
            let icon =  match current_operation.as_ref() {
                Some(op) => {
                    match op {
                        LayerOperation::Difference => {icons::_LAYERDIFFERENCE}
                        LayerOperation::Union => {icons::_LAYERUNION}
                        LayerOperation::XOR => {icons::_LAYERXOR}
                        LayerOperation::Intersect => {icons::_LAYERINTERSECTION}
                    }
                }
                None => {icons::LAYERCOMBINE}
            };
            ui.button(unsafe { imgui::ImStr::from_utf8_with_nul_unchecked(icon) }, [0., 0.]);
            if ui.is_item_clicked(imgui::MouseButton::Right) {
                let active_layer = v.get_active_layer();
                v.set_active_layer(layer);
                v.begin_layer_modification("Changed layer operation.");
                v.with_active_layer_mut(|layer| {
                    layer.operation = None;
                });
                v.end_layer_modification();
                v.set_active_layer(active_layer);
            }
            if ui.is_item_clicked(imgui::MouseButton::Left) {
                let new_operation = match current_operation {
                    Some(op) => {
                        match op {
                            LayerOperation::Difference => { Some(LayerOperation::Union) }
                            LayerOperation::Union => { Some(LayerOperation::XOR) }
                            LayerOperation::XOR => { Some(LayerOperation::Intersect)}
                            LayerOperation::Intersect => { None }
                        }
                    }
                    None => { Some(LayerOperation::Difference) }
                };
    
                let active_layer = v.get_active_layer();
                v.set_active_layer(layer);
                v.begin_layer_modification("Changed layer operation.");
                v.with_active_layer_mut(|layer| {
                    layer.operation = new_operation.clone();
                });
                v.end_layer_modification();
                v.set_active_layer(active_layer);
            }
    
            if layer_op.is_none() {
                ui.same_line(0.);
                let mut color_token: Option<ColorStackToken> = None;
                let _default_color: Option<[f32; 4]> = None;
                if let Some(color) = v.with_glyph(|glif| glif.layers[layer].color) {
                    color_token = Some(ui.push_style_color(imgui::StyleColor::Button, color.into()));
                }
                ui.button(imgui::im_str!("##"), [0., 0.]);
                if ui.is_item_clicked(imgui::MouseButton::Left) {
                    i.push_prompt(InputPrompt::Color {
                        label: "Layer color:".to_string(),
                        default: v.with_glyph(|glif| glif.layers[layer].color.unwrap_or([0., 0., 0., 1.].into())).into(),
                        func: Rc::new(move |editor, color| {
                            let active_layer = editor.get_active_layer();
                            editor.set_active_layer(layer);
            
                            editor.begin_layer_modification("Changed layer color.");
                            editor.with_active_layer_mut(|layer| layer.color = color.map(|c|c.into()));
                            editor.end_layer_modification();
            
                            editor.set_active_layer(active_layer);
                        }),
                    });
                }
    
                if let Some(token) = color_token {
                    token.pop(ui);
                }
            }
    
            font_token.pop(ui);
            custom_button_color.pop(ui);
    
            ui.same_line(0.);
            let mut pop_me = None;
            if active_layer != layer {
                pop_me = Some(ui.push_style_color(imgui::StyleColor::Button, [0., 0., 0., 0.2]));
            }
            ui.button(&im_str, [-1., 0.]);
            if ui.is_item_clicked(imgui::MouseButton::Left) {
                v.set_active_layer(layer);
            }
            if let Some(p) = pop_me {
                p.pop(ui);
            }
            no_padding.pop(ui);
        }
    }
    
    pub fn build_imgui_ui<'ui>(context: &'ui mut Context, imsdl2: &mut ImguiSdl2, v: &mut Editor, i: &mut Interface, mouse_state: &MouseState) -> &'ui DrawData {
        imsdl2.prepare_frame(context.io_mut(), &i.sdl_window, mouse_state);
        let mut ui = context.frame();

        imgui::Window::new(imgui::im_str!("Tools"))
            .bg_alpha(1.) // See comment on fn redraw_skia
            .flags(
                        imgui::WindowFlags::NO_RESIZE
                    | imgui::WindowFlags::NO_MOVE
                    | imgui::WindowFlags::NO_COLLAPSE,
            )
            .position(
                [TOOLBOX_OFFSET_X, TOOLBOX_OFFSET_Y],
                imgui::Condition::Always,
            )
            .size([TOOLBOX_WIDTH, *TOOLBOX_HEIGHT+90.], imgui::Condition::Always)
            .build(&ui, || {
                Self::build_and_check_button(v, &ui, ToolEnum::Pan, &icons::PAN);
                Self::build_and_check_button(v, &ui, ToolEnum::Select, &icons::SELECT);
                ui.separator();
                Self::build_and_check_button(v, &ui, ToolEnum::Zoom, &icons::ZOOM);
                ui.separator();
                Self::build_and_check_button(v, &ui, ToolEnum::Anchors, &icons::ANCHOR);
                ui.separator();
                Self::build_and_check_button(v, &ui, ToolEnum::Pen, &icons::PEN);
                Self::build_and_check_button(v, &ui, ToolEnum::VWS, &icons::VWS);
                Self::build_and_check_button(v, &ui, ToolEnum::PAP, &icons::_PAP);
                Self::build_and_check_button(v, &ui, ToolEnum::Shapes, &icons::SHAPES);
                Self::build_and_check_button(v, &ui, ToolEnum::Grid, &icons::GRID);
                Self::build_and_check_button(v, &ui, ToolEnum::Image, &icons::IMAGES);
                Self::build_and_check_button(v, &ui, ToolEnum::Guidelines, &icons::GUIDELINES);
            });
    
        imgui::Window::new( imgui::im_str!("Layers"))
            .bg_alpha(1.)
            .flags(
                        imgui::WindowFlags::NO_RESIZE
                    | imgui::WindowFlags::NO_MOVE
                    | imgui::WindowFlags::NO_COLLAPSE
            )
            .position([i.viewport.winsize.0 as f32 - LAYERBOX_WIDTH - TOOLBOX_OFFSET_X , i.viewport.winsize.1 as f32 - TOOLBOX_OFFSET_Y - LAYERBOX_HEIGHT], imgui::Condition::Always)
            .size([LAYERBOX_WIDTH, LAYERBOX_HEIGHT], imgui::Condition::Always)
            .build(&ui, || {
                Self::build_and_check_layer_list(v, i, &ui)
            });
    
            Self::build_and_check_prompts(v, i, &mut ui);
    
        v.dispatch_editor_event(i, EditorEvent::Ui {
            ui: &mut ui
        });

        imsdl2.prepare_render(&ui, &i.sdl_window);
        return ui.render();
    }
    
    fn build_and_check_prompts(v: &mut Editor, i: &mut Interface, ui: &mut imgui::Ui)
    {
        if !i.active_prompts() { return };
    
        imgui::Window::new(&imgui::im_str!("##"))
        .flags(
                    imgui::WindowFlags::NO_RESIZE
                | imgui::WindowFlags::NO_MOVE
                | imgui::WindowFlags::NO_COLLAPSE
                | imgui::WindowFlags::NO_DECORATION
                | imgui::WindowFlags::NO_BACKGROUND
        )
        .position([0., 0.], imgui::Condition::Always)
        .size([i.viewport.winsize.0 as f32, i.viewport.winsize.1 as f32], imgui::Condition::Always)
        .build(ui, || {
            ui.invisible_button(&imgui::im_str!("##"), [-1., -1.]);
            if ui.is_item_clicked(imgui::MouseButton::Right) {
                i.pop_prompt();
            }
        });
            
        match i.peek_prompt().clone() {
            InputPrompt::Text{ label, default: _, func} => {
                imgui::Window::new(&imgui::im_str!("{}", label))
                .bg_alpha(1.) // See comment on fn redraw_skia
                .flags(
                            imgui::WindowFlags::NO_RESIZE
                        | imgui::WindowFlags::NO_COLLAPSE,
                )
                .position_pivot([0.5, 0.5])
                .position(
                    [(i.viewport.winsize.0/2) as f32, (i.viewport.winsize.1/2) as f32],
                    imgui::Condition::Always,
                )
                .size([*TOOLBOX_HEIGHT, TOOLBOX_WIDTH+10.], imgui::Condition::Always)
                .focused(true)
                .build(ui, || {
                    PROMPT_STR.with(|prompt_str| {
                        ui.push_item_width(-1.);
                        prompt_str.borrow_mut().clear();
                        ui.input_text(imgui::im_str!(""), &mut prompt_str.borrow_mut())
                        .build();
        
                        if ui.is_key_down(Key::Enter) {
                            let final_string = prompt_str.borrow().to_string();
                            let mut new_string = imgui::ImString::new("");
                            new_string.reserve(256);
                            prompt_str.replace(new_string);
                            func(v, final_string);
                            i.pop_prompt();
                        }
                    })
                });
            }
    
            InputPrompt::Color { label, default: _, func } => {
                let mut color = PROMPT_CLR.with(|clr| clr.borrow_mut().clone() );
    
                imgui::Window::new(&imgui::im_str!("{}", label))
                .bg_alpha(1.) // See comment on fn redraw_skia
                .flags(
                            imgui::WindowFlags::NO_RESIZE
                        | imgui::WindowFlags::NO_COLLAPSE,
                )
                .position_pivot([0.5, 0.5])
                .position(
                    [(i.viewport.winsize.0/2) as f32, (i.viewport.winsize.1/2) as f32],
                    imgui::Condition::Always,
                )
                .size([*TOOLBOX_HEIGHT, *TOOLBOX_HEIGHT + 10.], imgui::Condition::Always)
                .focused(true)
                .build(ui, || {
                    PROMPT_CLR.with(|ui_color| {
                        imgui::ColorPicker::new(&imgui::im_str!("{}", label), &mut color)
                        .build(ui);        
        
                        if ui.is_key_down(Key::Enter) {
                            ui_color.replace([0., 0., 0., 1.]);
                            func(v, Some(color));
                            i.pop_prompt();
                        }
    
                        ui.button(imgui::im_str!("Automatic"), [0., 0.]);
                        if ui.is_item_clicked(imgui::MouseButton::Left) {
                            func(v, None);
                            i.pop_prompt();
                        }
                    })
                });
    
                PROMPT_CLR.with(|clr| clr.replace(color));
            }
    
            InputPrompt::Layer{ label, func} => {
                imgui::Window::new(&imgui::im_str!("{}", label))
                .bg_alpha(1.) // See comment on fn redraw_skia
                .flags(
                        imgui::WindowFlags::NO_RESIZE
                        | imgui::WindowFlags::NO_COLLAPSE,
                )
                .position_pivot([0.5, 0.5])
                .position(
                    [(i.viewport.winsize.0/2) as f32, (i.viewport.winsize.1/2) as f32],
                    imgui::Condition::Always,
                )
                .size([*TOOLBOX_HEIGHT, *TOOLBOX_HEIGHT + 10.], imgui::Condition::Always)
                .focused(true)
                .build(ui, || {
                    let layer_count = v.with_glyph(|glif| glif.layers.len());
                    for layer in 0 .. layer_count {
                        let layer_op = v.with_glyph(|glif| glif.layers[layer].operation.clone());
                        let layer_temp_name = imgui::im_str!("{0}", v.with_glyph(|glif| { glif.layers[layer].name.clone() }));
                        let im_str = imgui::ImString::from(layer_temp_name);
                
                        let no_padding = ui.push_style_var(StyleVar::ItemSpacing([0., 0.]));
                
                        if layer_op.is_some() {
                            ui.dummy([28., 0.]);
                            ui.same_line(0.);
                        }
                
                        let mut pop_me = None;
                        if v.get_active_layer() != layer {
                            pop_me = Some(ui.push_style_color(imgui::StyleColor::Button, [0., 0., 0., 0.2]));
                        }
                        ui.button(&im_str, [-1., 0.]);
                        if ui.is_item_clicked(imgui::MouseButton::Left) {
                            func(v, v.with_glyph(|glif| glif.layers[layer].clone() ));
                            i.pop_prompt();
                        }
                        if let Some(p) = pop_me {
                            p.pop(ui);
                        }
                        no_padding.pop(ui);
                    }
                });
            }
        }
    }

    pub fn handle_imgui_event(&mut self, sdl_event: &Event) -> bool {
        self.imgui_sdl2.handle_event(&mut self.imgui_context, &sdl_event);
        if self.imgui_sdl2.ignore_event(sdl_event) {
            return true;
        };

        false
    }
}

thread_local! { pub static PROMPT_STR: RefCell<imgui::ImString> = RefCell::new(imgui::ImString::new("")); }
thread_local! { pub static PROMPT_CLR: RefCell<[f32; 4]> = RefCell::new([0., 0., 0., 1.]); }
thread_local! { pub static FONT_IDS: RefCell<Vec<FontId>> = RefCell::new(vec!()); }
