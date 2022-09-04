use super::state::InputState;

use crate::{
  camera::Camera,
  spaces::{CanvasVector, CanvasVectorExt},
};

pub fn update_translate_tool(input: &InputState, camera_screen: &mut Camera) {
  if !input.is_clicked(winit::event::MouseButton::Left) {
    return;
  }

  if let Some(cursor_diff) = input.cursor_screen_pixel_difference() {
    let cursor_diff = CanvasVector::from_screen_pixel(cursor_diff, camera_screen);
    camera_screen.position -= cursor_diff;
  }
}
