use crate::{spaces::ScreenPixelPointExt, tools::SelectLoop, ui::UiAccess};

pub fn ui_select_loop(ui: &egui::Ui, ui_access: &mut UiAccess) {
  let screen_points = match &ui_access.tool_manager.select_loop {
    SelectLoop::Selecting { screen_points } => screen_points,
    _ => return,
  };

  let painter = ui.painter();
  let stroke = egui::Stroke::new(3.0, egui::Color32::DARK_BLUE);
  let screen_points = screen_points
    .iter()
    .map(|p| p.into_egui(ui_access.camera))
    .collect();
  let line = egui::Shape::closed_line(screen_points, stroke);
  painter.add(line);
}
