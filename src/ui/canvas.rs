use super::{overlay::ui_overlay, UiAccess};

#[derive(Default)]
pub struct CanvasUi {
  has_focus: bool,
}

impl CanvasUi {
  pub fn ui(&mut self, ctx: &egui::Context, ui_access: &mut UiAccess) {
    egui::CentralPanel::default()
      .frame(egui::Frame::canvas(&ctx.style()).fill(egui::Color32::TRANSPARENT))
      .show(ctx, |ui| {
        let rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(rect, egui::Sense::hover());
        ui_access.camera.viewport = rect;
        self.has_focus = response.hovered();

        ui_overlay(ctx, ui_access);
      });
  }

  pub fn has_focus(&self) -> bool {
    self.has_focus
  }
}
