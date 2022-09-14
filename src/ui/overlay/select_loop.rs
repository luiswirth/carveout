use crate::{spaces::Space, tools::SelectLoop, ui::UiAccess};

pub fn ui_select_loop(ui: &egui::Ui, ui_access: &mut UiAccess) {
  let points_screen_logical = match &ui_access.tool_manager.select_loop {
    SelectLoop::Selecting {
      points_screen_logical,
    } => points_screen_logical,
    _ => return,
  };

  let painter = ui.painter();
  let stroke = egui::Stroke::new(3.0, egui::Color32::DARK_BLUE);
  let screen_points = points_screen_logical
    .iter()
    .map(|p| {
      ui_access
        .spaces
        .transform_point(*p, Space::ScreenLogical, Space::WindowLogical)
    })
    .map(|p| egui::Pos2::new(p.x, p.y))
    .collect();
  let line = egui::Shape::closed_line(screen_points, stroke);
  painter.add(line);
}
