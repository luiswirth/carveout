use crate::util::{SpacePoint, SpaceUnit, SpaceVector};

use super::gfx::CameraWithScreen;

/// Space representing the pixelated canvas screen
/// One Unit is one logical canvas screen pixel, therefore square.
/// Origin is top left corner of screen.
pub struct ScreenPixelSpace;
pub type ScreenPixelUnit = SpaceUnit<ScreenPixelSpace>;
pub type ScreenPixelPoint = SpacePoint<ScreenPixelSpace>;
pub type ScreenPixelVector = SpaceVector<ScreenPixelSpace>;

/// Space representing the normalized canvas screen
/// One Unit is the whole screen along an axis, therefore not square.
/// Origin is top left corner of screen.
pub struct ScreenNormalizedSpace;
//pub type ScreenNormalizedUnit = SpaceUnit<ScreenNormalizedSpace>;
//pub type ScreenNormalizedPoint = SpacePoint<ScreenNormalizedSpace>;
pub type ScreenNormalizedVector = SpaceVector<ScreenNormalizedSpace>;

/// Space representing canvas space.
pub struct CanvasSpace;
//pub type CanvasUnit = SpaceUnit<CanvasSpace>;
pub type CanvasPoint = SpacePoint<CanvasSpace>;
pub type CanvasVector = SpaceVector<CanvasSpace>;

pub trait ScreenPixelPointExt
where
  Self: Sized,
{
  fn from_window_logical(
    point: winit::dpi::LogicalPosition<f64>,
    camera_screen: &CameraWithScreen,
  ) -> Self;
  fn try_from_window_logical(
    point: winit::dpi::LogicalPosition<f64>,
    camera_screen: &CameraWithScreen,
  ) -> Option<Self>;
  fn from_canvas(point: CanvasPoint, camera_screen: &CameraWithScreen) -> Self;
}
impl ScreenPixelPointExt for ScreenPixelPoint {
  fn from_window_logical(
    point: winit::dpi::LogicalPosition<f64>,
    camera_screen: &CameraWithScreen,
  ) -> Self {
    let point = point.cast::<f32>();
    let screen_min = camera_screen.screen_rect().min;
    na::Point2::new(point.x - screen_min.x, point.y - screen_min.y).cast()
  }

  fn try_from_window_logical(
    point: winit::dpi::LogicalPosition<f64>,
    camera_screen: &CameraWithScreen,
  ) -> Option<Self> {
    match camera_screen
      .screen_rect()
      .contains(egui::Pos2::new(point.x as f32, point.y as f32))
    {
      true => Some(Self::from_window_logical(point, camera_screen)),
      false => None,
    }
  }

  fn from_canvas(canvas_point: CanvasPoint, camera_screen: &CameraWithScreen) -> Self {
    let canvas_point = canvas_point.cast();
    let view_point = camera_screen
      .view_transform()
      .transform_point(&canvas_point);
    let normalized_screen_point = camera_screen.projection().transform_point(&view_point);
    let screen_point = camera_screen
      .screen_transform()
      .transform_point(&normalized_screen_point);
    screen_point.cast()
  }
}

pub trait ScreenNormalizedVectorExt {
  fn from_pixel(vector: ScreenPixelVector, camera_screen: &CameraWithScreen) -> Self;
}
impl ScreenNormalizedVectorExt for ScreenNormalizedVector {
  fn from_pixel(vector: ScreenPixelVector, camera_screen: &CameraWithScreen) -> Self {
    let screen_size = camera_screen.screen_rect().size();
    na::Vector2::new(vector.x.0 / screen_size.x, vector.y.0 / screen_size.y).cast()
  }
}

pub trait CanvasPointExt {
  fn from_screen(point: ScreenPixelPoint, camera_screen: &CameraWithScreen) -> Self;
}
impl CanvasPointExt for CanvasPoint {
  fn from_screen(screen_point: ScreenPixelPoint, camera_screen: &CameraWithScreen) -> Self {
    let screen_point = screen_point.cast();
    let normalized_screen_point = camera_screen
      .screen_transform()
      .inverse_transform_point(&screen_point);
    let view_point = camera_screen
      .projection()
      .try_inverse_transform_point(&normalized_screen_point)
      .unwrap();
    let canvas_point = camera_screen
      .view_transform()
      .inverse_transform_point(&view_point);
    canvas_point.cast()
  }
}

pub trait CanvasVectorExt {
  fn from_screen(screen_vector: ScreenPixelVector, cavnas_gfx: &CameraWithScreen) -> Self;
}
impl CanvasVectorExt for CanvasVector {
  fn from_screen(screen_vector: ScreenPixelVector, camera_screen: &CameraWithScreen) -> Self {
    let screen_vector = screen_vector.cast();
    let normalized_screen_vector = camera_screen
      .screen_transform()
      .inverse_transform_vector(&screen_vector);
    let view_vector = camera_screen.projection().try_inverse().unwrap() * normalized_screen_vector;
    let canvas_vector = camera_screen
      .view_transform()
      .inverse_transform_vector(&view_vector);
    canvas_vector.cast()
  }
}
