use winit::window::Window;

use super::gfx::CameraWithScreen;

use crate::util::{SpacePoint, SpaceUnit, SpaceVector};

/// Space representing the pixelated canvas screen.
/// One Unit is one logical canvas screen pixel, therefore square.
/// Origin is top left corner of screen.
pub struct ScreenPixelSpace;
pub type ScreenPixelUnit = SpaceUnit<ScreenPixelSpace>;
pub type ScreenPixelPoint = SpacePoint<ScreenPixelSpace>;
pub type ScreenPixelVector = SpaceVector<ScreenPixelSpace>;

/// Space representing the normalized canvas screen.
/// One Unit is half the screen along an axis, therefore not square.
/// Origin is the center the of screen.
/// This is the same as normalized device coordinates or clip space.
pub struct ScreenNormSpace;
//pub type ScreenNormUnit = SpaceUnit<ScreenNormSpace>;
pub type ScreenNormPoint = SpacePoint<ScreenNormSpace>;
pub type ScreenNormVector = SpaceVector<ScreenNormSpace>;

/// Space representing canvas space.
pub struct CanvasSpace;
//pub type CanvasUnit = SpaceUnit<CanvasSpace>;
pub type CanvasPoint = SpacePoint<CanvasSpace>;
pub type CanvasVector = SpaceVector<CanvasSpace>;

pub trait ScreenPixelPointExt
where
  Self: Sized,
{
  fn is_in_screen(&self, camera_screen: &CameraWithScreen) -> bool;

  fn from_window_logical(
    point: winit::dpi::LogicalPosition<f64>,
    camera_screen: &CameraWithScreen,
  ) -> Self;
  fn from_canvas(point: CanvasPoint, camera_screen: &CameraWithScreen) -> Self;
}
impl ScreenPixelPointExt for ScreenPixelPoint {
  fn is_in_screen(&self, camera_screen: &CameraWithScreen) -> bool {
    let size = camera_screen.screen_rect().size();
    self.x.0 >= 0.0 && self.y.0 >= 0.0 && self.x.0 < size.x && self.y.0 < size.y
  }

  fn from_window_logical(
    point: winit::dpi::LogicalPosition<f64>,
    camera_screen: &CameraWithScreen,
  ) -> Self {
    let point = point.cast::<f32>();
    let screen_min = camera_screen.screen_rect().min;
    na::Point2::new(point.x - screen_min.x, point.y - screen_min.y).cast()
  }

  fn from_canvas(canvas_point: CanvasPoint, camera_screen: &CameraWithScreen) -> Self {
    let canvas_point = canvas_point.cast();
    let view_point = camera_screen
      .canvas_to_view()
      .transform_point(&canvas_point);
    let screen_norm_point = camera_screen
      .view_to_screen_norm()
      .transform_point(&view_point);
    let screen_pixel_point = camera_screen
      .screen_norm_to_pixel()
      .transform_point(&screen_norm_point);
    screen_pixel_point.cast()
  }
}

pub trait ScreenPixelVectorExt {
  fn from_window_logical(point: winit::dpi::LogicalPosition<f64>) -> Self;

  fn from_canvas(canvas_vector: CanvasVector, camera_screen: &CameraWithScreen) -> Self;
}
impl ScreenPixelVectorExt for ScreenPixelVector {
  fn from_window_logical(point: winit::dpi::LogicalPosition<f64>) -> Self {
    let point = point.cast::<f32>();
    na::Vector2::new(point.x, point.y).cast()
  }

  fn from_canvas(canvas_vector: CanvasVector, camera_screen: &CameraWithScreen) -> Self {
    let canvas_vector = canvas_vector.cast();
    let view_vector = camera_screen
      .canvas_to_view()
      .transform_vector(&canvas_vector);
    let screen_norm_vector = camera_screen.view_to_screen_norm() * view_vector;
    let screen_pixel_vector = camera_screen
      .screen_norm_to_pixel()
      .transform_vector(&screen_norm_vector);
    screen_pixel_vector.cast()
  }
}

pub trait ScreenNormPointExt {
  fn from_screen_pixel(
    screen_pixel_point: ScreenPixelPoint,
    camera_screen: &CameraWithScreen,
  ) -> Self;
}
impl ScreenNormPointExt for ScreenNormPoint {
  fn from_screen_pixel(
    screen_pixel_point: ScreenPixelPoint,
    camera_screen: &CameraWithScreen,
  ) -> Self {
    camera_screen
      .screen_norm_to_pixel()
      .inverse_transform_point(&screen_pixel_point.cast())
      .cast()
  }
}

pub trait ScreenNormVectorExt {
  fn from_screen_pixel(
    screen_pixel_vector: ScreenPixelVector,
    camera_screen: &CameraWithScreen,
  ) -> Self;
}
impl ScreenNormVectorExt for ScreenNormVector {
  fn from_screen_pixel(
    screen_pixel_vector: ScreenPixelVector,
    camera_screen: &CameraWithScreen,
  ) -> Self {
    camera_screen
      .screen_norm_to_pixel()
      .inverse_transform_vector(&screen_pixel_vector.cast())
      .cast()
  }
}

pub trait CanvasPointExt {
  fn from_screen_pixel(
    screen_pixel_point: ScreenPixelPoint,
    camera_screen: &CameraWithScreen,
  ) -> Self;
}
impl CanvasPointExt for CanvasPoint {
  fn from_screen_pixel(
    screen_pixel_point: ScreenPixelPoint,
    camera_screen: &CameraWithScreen,
  ) -> Self {
    let screen_pixel_point = screen_pixel_point.cast();
    let screen_norm_point = camera_screen
      .screen_norm_to_pixel()
      .inverse_transform_point(&screen_pixel_point);
    let view_point = camera_screen
      .view_to_screen_norm()
      .try_inverse_transform_point(&screen_norm_point)
      .unwrap();
    let canvas_point = camera_screen
      .canvas_to_view()
      .inverse_transform_point(&view_point);
    canvas_point.cast()
  }
}

pub trait CanvasVectorExt {
  fn from_screen_pixel(
    screen_pixel_vector: ScreenPixelVector,
    camera_screen: &CameraWithScreen,
  ) -> Self;
  fn from_screen_norm(
    screen_norm_vector: ScreenNormVector,
    camera_screen: &CameraWithScreen,
  ) -> Self;
}
impl CanvasVectorExt for CanvasVector {
  fn from_screen_pixel(
    screen_pixel_vector: ScreenPixelVector,
    camera_screen: &CameraWithScreen,
  ) -> Self {
    let screen_pixel_vector = screen_pixel_vector.cast();
    let screen_norm_vector = camera_screen
      .screen_norm_to_pixel()
      .inverse_transform_vector(&screen_pixel_vector);
    let view_vector =
      camera_screen.view_to_screen_norm().try_inverse().unwrap() * screen_norm_vector;
    let canvas_vector = camera_screen
      .canvas_to_view()
      .inverse_transform_vector(&view_vector);
    canvas_vector.cast()
  }

  fn from_screen_norm(
    screen_norm_vector: ScreenNormVector,
    camera_screen: &CameraWithScreen,
  ) -> Self {
    let screen_norm_vector = screen_norm_vector.cast();
    let view_vector =
      camera_screen.view_to_screen_norm().try_inverse().unwrap() * screen_norm_vector;
    let canvas_vector = camera_screen
      .canvas_to_view()
      .inverse_transform_vector(&view_vector);
    canvas_vector.cast()
  }
}

#[derive(Clone, Debug)]
pub struct PointInSpaces {
  pub screen_pixel: ScreenPixelPoint,
  pub screen_norm: ScreenNormPoint,
  pub canvas: CanvasPoint,
  pub in_screen: bool,
}
impl PointInSpaces {
  pub fn from_window_physical(
    window_physical: winit::dpi::PhysicalPosition<f64>,
    window: &Window,
    camera_screen: &CameraWithScreen,
  ) -> Self {
    let window_logical = window_physical.to_logical(window.scale_factor());
    let screen_pixel = ScreenPixelPoint::from_window_logical(window_logical, camera_screen);
    Self::from_screen_pixel(screen_pixel, camera_screen)
  }

  pub fn from_screen_pixel(
    screen_pixel: ScreenPixelPoint,
    camera_screen: &CameraWithScreen,
  ) -> Self {
    let screen_norm = ScreenNormPoint::from_screen_pixel(screen_pixel, camera_screen);
    let canvas = CanvasPoint::from_screen_pixel(screen_pixel, camera_screen);
    let in_screen = screen_pixel.is_in_screen(camera_screen);

    Self {
      screen_pixel,
      screen_norm,
      canvas,
      in_screen,
    }
  }
}

#[derive(Clone, Debug)]
pub struct VectorInSpaces {
  pub screen_pixel: ScreenPixelVector,
  pub screen_norm: ScreenNormVector,
  pub canvas: CanvasVector,
}
impl VectorInSpaces {
  pub fn from_screen_pixel(
    screen_pixel: ScreenPixelVector,
    camera_screen: &CameraWithScreen,
  ) -> Self {
    let screen_norm = ScreenNormVector::from_screen_pixel(screen_pixel, camera_screen);
    let canvas = CanvasVector::from_screen_pixel(screen_pixel, camera_screen);
    Self {
      screen_pixel,
      screen_norm,
      canvas,
    }
  }
}
