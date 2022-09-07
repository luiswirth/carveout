use crate::{camera::Camera, input::InputManager};

pub fn update_zoom_tool(input: &InputManager, camera: &mut Camera) {
  if let Some(clicked_cursor_pos) = &input.cursor_pos_left_clicked {
    if let Some(cursor_norm_diff) = input.cursor_screen_norm_difference() {
      let scale_factor = 1.0 + cursor_norm_diff.y.0;
      camera.zoom_with_center(scale_factor, clicked_cursor_pos.screen_pixel);
    }
  }
}
