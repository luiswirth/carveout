mod indicators;
mod select_loop;

use self::{indicators::ui_indicators, select_loop::ui_select_loop};

use super::UiAccess;

pub fn ui_overlay(ctx: &egui::Context, ui_access: &mut UiAccess, screen_rect: egui::Rect) {
  let layer_id = egui::LayerId::new(egui::Order::Background, egui::Id::new("canvas_ui_overlay"));
  let ui = egui::Ui::new(ctx.clone(), layer_id, layer_id.id, screen_rect, screen_rect);

  ui_select_loop(&ui, ui_access);
  ui_indicators(&ui, ui_access, screen_rect);
}
