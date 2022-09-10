#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
  pub position_canvas: na::Point2<f32>,
  pub angle: f32,
  pub zoom: f32,
}
impl Default for Camera {
  fn default() -> Self {
    let position_canvas = na::Point2::origin();
    let angle = 0.0;
    let zoom = 1.0;

    Self {
      position_canvas,
      angle,
      zoom,
    }
  }
}

impl Camera {
  pub fn rotate_with_center(&mut self, angle: f32, center_canvas: na::Point2<f32>) {
    let mut vector_canvas = self.position_canvas - center_canvas;
    let rotation = na::Rotation2::new(angle);
    vector_canvas = rotation.transform_vector(&vector_canvas);

    self.position_canvas = center_canvas + vector_canvas;
    self.angle = (self.angle + angle).rem_euclid(std::f32::consts::TAU);
  }

  pub fn zoom_with_center(&mut self, zoom: f32, center_canvas: na::Point2<f32>) {
    let vector_canvas = (self.position_canvas - center_canvas).scale(1.0 / zoom);

    self.position_canvas = center_canvas + vector_canvas;
    self.zoom *= zoom;
  }
}

pub mod controller {
  use crate::{
    input::{InputManager, TouchMovement},
    spaces::{Space, SpaceManager},
  };

  use winit::event::VirtualKeyCode;

  pub fn update(spaces: &mut SpaceManager, input: &InputManager) {
    movement_key(spaces, input);
    movement_mouse(spaces, input);
    movement_touch(spaces, input);
  }
  fn movement_key(spaces: &mut SpaceManager, input: &InputManager) {
    let mut translation_screen_norm = na::Vector2::zeros();
    const TRANSLATION_SPEED: f32 = 1.0 / 35.0;
    if input.is_pressed(VirtualKeyCode::W) {
      translation_screen_norm.y += TRANSLATION_SPEED;
    }
    if input.is_pressed(VirtualKeyCode::A) {
      translation_screen_norm.x += TRANSLATION_SPEED;
    }
    if input.is_pressed(VirtualKeyCode::S) {
      translation_screen_norm.y -= TRANSLATION_SPEED;
    }
    if input.is_pressed(VirtualKeyCode::D) {
      translation_screen_norm.x -= TRANSLATION_SPEED;
    }

    let mut angle = 0.0;
    const ROTATION_SPEED: f32 = 0.04;
    if input.is_pressed(VirtualKeyCode::Q) {
      angle -= ROTATION_SPEED;
    }
    if input.is_pressed(VirtualKeyCode::E) {
      angle += ROTATION_SPEED;
    }

    let mut scale = 1.0;
    const SCALE_SPEED: f32 = 0.01;
    if input.is_pressed(VirtualKeyCode::Space) {
      scale -= SCALE_SPEED;
    }
    if input.is_pressed(VirtualKeyCode::LShift) {
      scale += SCALE_SPEED;
    }

    if translation_screen_norm != na::Vector2::zeros() {
      let translation_canvas =
        spaces.transform_vector(translation_screen_norm, Space::ScreenNorm, Space::Canvas);
      spaces.camera_mut().position_canvas -= translation_canvas;
    }
    let camera = spaces.camera_mut();
    camera.angle += angle;
    camera.zoom *= scale;
  }

  fn movement_mouse(spaces: &mut SpaceManager, input: &InputManager) {
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

    let mut translation_screen_norm = na::Vector2::zeros();
    let mut angle = 0.0;
    let mut scale = 1.0;

    if let Some(scroll_delta_logical) = input.mouse_scroll_delta_logical {
      let scroll_delta_screen_norm = spaces.transform_vector(
        scroll_delta_logical,
        Space::ScreenLogical,
        Space::ScreenNorm,
      );
      match meaning {
        ScrollMeaning::Translation => {
          const TRANSLATION_SPEED: f32 = 5.0;
          translation_screen_norm += scroll_delta_screen_norm.scale(TRANSLATION_SPEED);
        }
        ScrollMeaning::Rotation => {
          const ROTATION_SPEED: f32 = 10.0;
          angle += scroll_delta_screen_norm.y * ROTATION_SPEED;
        }
        ScrollMeaning::Scale => {
          const SCALE_SPEED: f32 = 3.0;
          scale += scroll_delta_screen_norm.y * SCALE_SPEED;
        }
      }
    }

    if translation_screen_norm != na::Vector2::zeros() {
      let translation_canvas =
        spaces.transform_vector(translation_screen_norm, Space::ScreenNorm, Space::Canvas);
      spaces.camera_mut().position_canvas -= translation_canvas;
    }
    if let Some(center_screen_logical) = input.curr.cursor_pos_screen_logical {
      let center_canvas =
        spaces.transform_point(center_screen_logical, Space::ScreenLogical, Space::Canvas);
      let camera = spaces.camera_mut();
      camera.rotate_with_center(angle, center_canvas);
      camera.zoom_with_center(scale, center_canvas);
    }
  }

  fn movement_touch(spaces: &mut SpaceManager, input: &InputManager) {
    if let Some(TouchMovement {
      center_screen_logical,
      translation_screen_logical,
      rotation,
      scale,
    }) = input.multi_touch_movement
    {
      let center_canvas =
        spaces.transform_point(center_screen_logical, Space::ScreenLogical, Space::Canvas);
      let translation_canvas = spaces.transform_vector(
        translation_screen_logical,
        Space::ScreenLogical,
        Space::Canvas,
      );
      let camera = spaces.camera_mut();
      camera.position_canvas -= translation_canvas;
      camera.rotate_with_center(-rotation, center_canvas);
      camera.zoom_with_center(scale, center_canvas);
    }
  }
}
