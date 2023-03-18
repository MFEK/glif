use super::EguiManager;
use egui::epaint;

const NO_SHADOW: epaint::Shadow = epaint::Shadow::NONE;

impl EguiManager {
    pub fn no_drop_shadows(&mut self) {
        let ctx = &self.egui.egui_ctx;
        let mut visuals = ctx.style().visuals.clone();
        visuals.window_shadow = NO_SHADOW;
        visuals.popup_shadow = NO_SHADOW;
        ctx.set_visuals(visuals);
    }
}
