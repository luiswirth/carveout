use crate::ui::UiAccess;

pub fn ui_indicators(ui: &egui::Ui, ui_access: &mut UiAccess, screen: egui::Rect) {
  let painter = ui.painter();

  let width = 3.0;
  let direction = -ui_access.camera.angle;
  let direction = egui::Vec2::angled(direction);
  let length = 50.0;

  let anchor = screen.min + egui::Vec2::splat(20.0) + egui::Vec2::splat(length);

  arrow(
    painter,
    anchor,
    direction,
    length,
    egui::Stroke::new(width, egui::Color32::RED),
    "X",
  );

  arrow(
    painter,
    anchor,
    direction.rot90(),
    length,
    egui::Stroke::new(width, egui::Color32::BLUE),
    "Y",
  );
}

fn arrow(
  painter: &egui::Painter,
  anchor: egui::Pos2,
  direction: egui::Vec2,
  length: f32,
  stroke: egui::Stroke,
  text: &str,
) {
  let arrow_head_length = 10.0;
  let arrow_head_width = 5.0;

  let start = anchor;
  let end = start + direction * length;

  let cross = direction.rot90() * arrow_head_width;
  let arrow_head_start = end - direction * arrow_head_length;

  painter.line_segment([start, arrow_head_start], stroke);
  painter.add(egui::Shape::convex_polygon(
    vec![arrow_head_start - cross, arrow_head_start + cross, end],
    stroke.color,
    egui::Stroke::none(),
  ));
  painter.text(
    end + 10.0 * direction,
    egui::Align2::CENTER_CENTER,
    text,
    egui::FontId::default(),
    stroke.color,
  );
}
