use crate::{
  input::InputManager,
  spaces::{Space, SpaceManager},
};

pub fn update_rotate_tool(input: &InputManager, spaces: &mut SpaceManager) {
  if let Some(center_screen_logical) = input.cursor_pos_screen_logical_left_clicked {
    let center_canvas =
      spaces.transform_point(center_screen_logical, Space::ScreenLogical, Space::Canvas);
    if let Some(cursor_logical_difference) = input.cursor_screen_logical_difference() {
      let cursor_norm_difference = spaces.transform_vector(
        cursor_logical_difference,
        Space::ScreenLogical,
        Space::ScreenNorm,
      );
      let rotation = cursor_norm_difference.x * std::f32::consts::PI;

      spaces
        .camera_mut()
        .rotate_with_center(rotation, center_canvas);
    }
  }
}
