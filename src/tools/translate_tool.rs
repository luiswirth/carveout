use crate::{
  input::InputManager,
  spaces::{Space, SpaceManager},
};

pub fn update_translate_tool(input: &InputManager, spaces: &mut SpaceManager) {
  if !input.is_clicked(winit::event::MouseButton::Left) {
    return;
  }

  if let Some(cursor_diff) = input.cursor_screen_logical_difference() {
    let cursor_diff = spaces.transform_vector(cursor_diff, Space::ScreenLogical, Space::Canvas);
    spaces.camera_mut().position_canvas -= cursor_diff;
  }
}
