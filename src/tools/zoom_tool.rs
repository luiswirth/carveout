use crate::{
  input::InputManager,
  spaces::{Space, SpaceManager},
};

pub fn update_zoom_tool(input: &InputManager, spaces: &mut SpaceManager) {
  if let Some(center_screen_logical) = input.cursor_pos_screen_logical_left_clicked {
    let center_canvas =
      spaces.transform_point(center_screen_logical, Space::ScreenLogical, Space::Canvas);
    if let Some(cursor_logical_diff) = input.cursor_screen_logical_difference() {
      let cursor_norm_diff =
        spaces.transform_vector(cursor_logical_diff, Space::ScreenLogical, Space::ScreenNorm);
      let scale_factor = 1.0 + cursor_norm_diff.y;
      spaces
        .camera_mut()
        .zoom_with_center(scale_factor, center_canvas);
    }
  }
}
