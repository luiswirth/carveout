use super::{overlay::ui_overlay, UiAccess};

pub struct CanvasUi {
  screen_rect: egui::Rect,
  has_focus: bool,
}
impl Default for CanvasUi {
  fn default() -> Self {
    Self {
      screen_rect: egui::Rect::NAN,
      has_focus: false,
    }
  }
}

impl CanvasUi {
  pub fn ui(&mut self, ctx: &egui::Context, ui_access: &mut UiAccess) {
    egui::CentralPanel::default()
      .frame(egui::Frame::canvas(&ctx.style()).fill(egui::Color32::TRANSPARENT))
      .show(ctx, |ui| {
        self.screen_rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(self.screen_rect, egui::Sense::hover());
        self.has_focus = response.hovered();

        ui_overlay(ctx, ui_access, self.screen_rect);
      });
  }

  pub fn has_focus(&self) -> bool {
    self.has_focus
  }

  pub fn screen_rect(&self) -> egui::Rect {
    self.screen_rect
  }
}
