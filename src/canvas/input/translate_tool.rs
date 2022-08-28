use crate::canvas::{
  gfx::CameraWithScreen,
  space::{CanvasVector, CanvasVectorExt},
};

use super::state::InputState;

pub fn update_translate_tool(input: &InputState, camera_screen: &mut CameraWithScreen) {
  if !input.is_clicked(winit::event::MouseButton::Left) {
    return;
  }

  if let Some(cursor_diff) = input.cursor_screen_pixel_difference() {
    let cursor_diff = CanvasVector::from_screen_pixel(cursor_diff, camera_screen);
    camera_screen.camera_mut().position -= cursor_diff;
  }
}
