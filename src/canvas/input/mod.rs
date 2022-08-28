mod state;

mod eraser;
mod pen;
mod rotate_tool;
mod scale_tool;
mod translate_tool;

use self::{
  eraser::update_eraser, rotate_tool::update_rotate_tool, scale_tool::update_scale_tool,
  state::InputState, translate_tool::update_translate_tool,
};

use super::{
  content::ContentManager,
  gfx::CameraWithScreen,
  space::{CanvasVector, CanvasVectorExt, ScreenNormVector},
  stroke::StrokeManager,
  tool::{ToolConfig, ToolEnum},
};

use crate::Event;

use winit::event::VirtualKeyCode;

#[derive(Default)]
pub struct InputHandler {
  state: InputState,

  pen_handler: self::pen::PenInputHandler,
}

impl InputHandler {
  pub fn handle_event(
    &mut self,
    event: &Event,
    window: &winit::window::Window,
    camera_screen: &mut CameraWithScreen,
  ) {
    if let Event::WindowEvent { event, window_id } = event {
      assert_eq!(*window_id, window.id());
      self.state.handle_event(event, window, camera_screen);
    }
  }

  pub fn update(
    &mut self,
    tool_config: &ToolConfig,
    content: &mut ContentManager,
    stroke_manager: &StrokeManager,
    camera_screen: &mut CameraWithScreen,
  ) {
    match tool_config.selected {
      ToolEnum::Pen => {
        self
          .pen_handler
          .update(&self.state, content, &tool_config.pen, camera_screen)
      }
      ToolEnum::Eraser => update_eraser(&self.state, content, stroke_manager),
      ToolEnum::Translate => update_translate_tool(&self.state, camera_screen),
      ToolEnum::Rotate => update_rotate_tool(&self.state, camera_screen),
      ToolEnum::Scale => update_scale_tool(&self.state, camera_screen),
    }

    Self::movement_key(&self.state, camera_screen);
    Self::movement_mouse(&self.state, camera_screen);

    self.state.update(camera_screen);
  }

  fn movement_key(input: &InputState, camera_screen: &mut CameraWithScreen) {
    let mut translation = ScreenNormVector::zeros();
    const TRANSLATION_SPEED: f32 = 1.0 / 25.0;
    if input.is_pressed(VirtualKeyCode::W) {
      translation.y += TRANSLATION_SPEED.into();
    }
    if input.is_pressed(VirtualKeyCode::A) {
      translation.x += TRANSLATION_SPEED.into();
    }
    if input.is_pressed(VirtualKeyCode::S) {
      translation.y -= TRANSLATION_SPEED.into();
    }
    if input.is_pressed(VirtualKeyCode::D) {
      translation.x -= TRANSLATION_SPEED.into();
    }

    let mut angle = 0.0;
    const ROTATION_SPEED: f32 = 0.05;
    if input.is_pressed(VirtualKeyCode::Q) {
      angle += ROTATION_SPEED;
    }
    if input.is_pressed(VirtualKeyCode::E) {
      angle -= ROTATION_SPEED;
    }

    let mut scale = 1.0;
    const SCALE_SPEED: f32 = 0.01;
    if input.is_pressed(VirtualKeyCode::Space) {
      scale -= SCALE_SPEED;
    }
    if input.is_pressed(VirtualKeyCode::LShift) {
      scale += SCALE_SPEED;
    }

    if translation != ScreenNormVector::zeros() {
      let translation = CanvasVector::from_screen_norm(translation, camera_screen);
      camera_screen.camera_mut().position -= translation;
    }
    camera_screen.camera_mut().angle += angle;
    camera_screen.camera_mut().scale *= scale;
  }

  fn movement_mouse(input: &InputState, camera_screen: &mut CameraWithScreen) {
    enum ScrollMeaning {
      Translation,
      Rotation,
      Scale,
    }
    let meaning = if input.is_pressed(VirtualKeyCode::LControl) {
      ScrollMeaning::Scale
    } else if input.is_pressed(VirtualKeyCode::LAlt) {
      ScrollMeaning::Rotation
    } else {
      ScrollMeaning::Translation
    };

    let mut translation = ScreenNormVector::zeros();
    let mut angle = 0.0;
    let mut scale = 1.0;

    match meaning {
      ScrollMeaning::Translation => {
        const TRANSLATION_SPEED: f32 = 5.0;
        if let Some(scroll_delta) = &input.mouse_scroll_delta {
          translation += scroll_delta.screen_norm.scale(TRANSLATION_SPEED.into());
        }
      }
      ScrollMeaning::Rotation => {
        const ROTATION_SPEED: f32 = 10.0;
        if let Some(scroll_delta) = &input.mouse_scroll_delta {
          angle += scroll_delta.screen_norm.y.0 * ROTATION_SPEED;
        }
      }
      ScrollMeaning::Scale => {
        const SCALE_SPEED: f32 = 3.0;
        if let Some(scroll_delta) = &input.mouse_scroll_delta {
          scale += scroll_delta.screen_norm.y.0 * SCALE_SPEED;
        }
      }
    }

    if translation != ScreenNormVector::zeros() {
      let translation = CanvasVector::from_screen_norm(translation, camera_screen);
      camera_screen.camera_mut().position -= translation;
    }
    if let Some(cursor) = &input.curr.cursor_pos {
      camera_screen.rotate_with_center(angle, cursor.screen_pixel);
      camera_screen.scale_with_center(scale, cursor.screen_pixel);
    }
  }
}
