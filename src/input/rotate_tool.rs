use super::state::InputState;

use crate::camera::Camera;

pub fn update_rotate_tool(input: &InputState, camera_screen: &mut Camera) {
  if let Some(clicked_cursor_pos) = &input.cursor_pos_left_clicked {
    if let Some(cursor_norm_diff) = input.cursor_screen_norm_difference() {
      let rotation = cursor_norm_diff.x.0 * std::f32::consts::PI;

      camera_screen.rotate_with_center(rotation, clicked_cursor_pos.screen_pixel);
    }
  }
}
