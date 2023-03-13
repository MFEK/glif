mod fonts;

use egui_skia::EguiSkia;
use sdl2::{event::Event, video::Window};

use super::Interface;

use egui_sdl2_event::{DpiMode, EguiSDL2State};

pub struct EguiManager {
    pub egui: EguiSkia,
    pub egui_sdl2: EguiSDL2State,
}

impl EguiManager {
    pub fn new(interface: &mut Interface) -> Self {
        let dpi = 1.0 / interface.set_dpi_from_os();
        let egui_sdl2 = egui_sdl2_event::EguiSDL2State::new(
            &interface.sdl_window,
            &interface.sdl_context.video().unwrap(),
            DpiMode::Custom(dpi),
        );
        let egui_skia = egui_skia::EguiSkia::new();
        return EguiManager {
            egui: egui_skia,
            egui_sdl2,
        };
    }

    pub fn init(&mut self) {
        self.set_system_fonts();
    }

    pub fn wants_event(&mut self, sdl_window: &Window, sdl_event: &Event) -> bool {
        self.egui_sdl2.sdl2_input_to_egui(sdl_window, sdl_event);

        // I don't think egui accepts all of these events, but it's a superset of the ones it does at least.
        match sdl_event {
            Event::KeyDown { .. }
            | Event::KeyUp { .. }
            | Event::TextEditing { .. }
            | Event::TextInput { .. } => self.egui.egui_ctx.wants_keyboard_input(),
            Event::MouseMotion { .. }
            | Event::MouseButtonDown { .. }
            | Event::MouseButtonUp { .. }
            | Event::MouseWheel { .. }
            | Event::FingerDown { .. }
            | Event::FingerUp { .. }
            | Event::FingerMotion { .. }
            | Event::DollarGesture { .. }
            | Event::DollarRecord { .. }
            | Event::MultiGesture { .. } => self.egui.egui_ctx.wants_pointer_input(),
            _ => false,
        }
    }
}
