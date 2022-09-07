use super::spaces::*;

#[derive(Debug, Clone)]
pub struct Camera {
  pub position: CanvasPoint,
  pub angle: f32,
  pub zoom: f32,
  pub viewport: egui::Rect,
}

impl Default for Camera {
  fn default() -> Self {
    let position = CanvasPoint::origin();
    let angle = 0.0;
    let zoom = 1.0;
    let viewport = egui::Rect::NAN;

    Self {
      position,
      angle,
      zoom,
      viewport,
    }
  }
}

impl Camera {
  // view transform (mVp)
  pub fn canvas_to_view(&self) -> na::IsometryMatrix2<f32> {
    let translation = na::Translation2::from(-self.position.cast());
    let rotation = na::Rotation2::new(-self.angle);
    rotation * translation
  }

  // projection (mvP)
  pub fn view_to_screen_norm(&self) -> na::Scale2<f32> {
    let camera_zoom = na::Scale2::new(self.zoom, self.zoom);
    let screen_scale = self.viewport.size();
    let screen_scale = na::Scale2::new(2.0 / screen_scale.x, 2.0 / screen_scale.y);
    screen_scale * camera_zoom
  }

  // viewport transform
  pub fn screen_norm_to_pixel(&self) -> na::Affine2<f32> {
    let translation = na::Translation2::new(1.0, 1.0);
    let translation: na::Affine2<f32> = na::convert(translation);
    let scale = self.viewport.size();
    let scale = na::Scale2::new(scale.x / 2.0, scale.y / 2.0);
    let scale: na::Affine2<f32> = na::convert(scale);
    scale * translation
  }
}
impl Camera {
  pub fn rotate_with_center(&mut self, angle: f32, center: ScreenPixelPoint) {
    let center = CanvasPoint::from_screen_pixel(center, self);
    let mut vector = self.position - center;
    let rotation = na::Rotation2::new(angle);
    vector = rotation.transform_vector(&vector.cast()).cast();

    self.position = center + vector;
    self.angle = (self.angle + angle).rem_euclid(std::f32::consts::TAU);
  }

  pub fn zoom_with_center(&mut self, zoom: f32, center: ScreenPixelPoint) {
    let center = CanvasPoint::from_screen_pixel(center, self);
    let mut vector = self.position - center;
    vector.scale_mut((1.0 / zoom).into());

    self.position = center + vector;
    self.zoom *= zoom;
  }
}

pub mod controller {
  use super::Camera;

  use crate::{
    input::InputManager,
    spaces::{CanvasVector, CanvasVectorExt, ScreenNormVector},
  };

  use winit::event::VirtualKeyCode;

  pub fn update(camera: &mut Camera, input: &InputManager) {
    movement_key(camera, input);
    movement_mouse(camera, input);
    movement_touch(camera, input);
  }

  fn movement_key(camera: &mut Camera, input: &InputManager) {
    let mut translation = ScreenNormVector::zeros();
    const TRANSLATION_SPEED: f32 = 1.0 / 35.0;
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

    if translation != ScreenNormVector::zeros() {
      let translation = CanvasVector::from_screen_norm(translation, camera);
      camera.position -= translation;
    }
    camera.angle += angle;
    camera.zoom *= scale;
  }

  fn movement_mouse(camera: &mut Camera, input: &InputManager) {
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
      let translation = CanvasVector::from_screen_norm(translation, camera);
      camera.position -= translation;
    }
    if let Some(cursor) = &input.curr.cursor_pos {
      camera.rotate_with_center(angle, cursor.screen_pixel);
      camera.zoom_with_center(scale, cursor.screen_pixel);
    }
  }

  fn movement_touch(camera: &mut Camera, input: &InputManager) {
    if let Some(movement) = &input.multi_touch_movement {
      camera.position -= movement.translation.canvas;
      camera.rotate_with_center(-movement.rotation, movement.center.screen_pixel);
      camera.zoom_with_center(movement.scale, movement.center.screen_pixel);
    }
  }
}
