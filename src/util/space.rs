use super::{SpacePoint, SpaceUnit, SpaceVector};

use crate::canvas::Camera;

pub struct Rect<S: 'static> {
  pub min: SpacePoint<S>,
  pub max: SpacePoint<S>,
}
impl<S> Default for Rect<S> {
  fn default() -> Self {
    Self {
      min: Default::default(),
      max: Default::default(),
    }
  }
}
impl<S: Send + Sync> Rect<S> {
  pub fn new(min: SpacePoint<S>, max: SpacePoint<S>) -> Self {
    Self { min, max }
  }

  pub fn center(&self) -> SpacePoint<S> {
    na::center(&self.min, &self.max).into()
  }

  pub fn size(&self) -> SpaceVector<S> {
    self.max - self.min
  }

  pub fn contains(&self, point: SpacePoint<S>) -> bool {
    self.min < point && point < self.max
  }
}

/// Same as [`winit::dpi::PhysicalPosition`]
/// One unit is a physical monitor pixel, therefore square
/// Origin is top left corner of window
pub struct WindowPhysicalSpace;
pub type WindowPhysicalPoint = SpacePoint<WindowPhysicalSpace>;

/// Same as [`winit::dpi::LogicalPosition`]
/// One unit is a logical monitor pixel, therefore square
/// Origin is top left corner of window
pub struct WindowLogicalSpace;
pub type WindowLogicalPoint = SpacePoint<WindowLogicalSpace>;
pub type WindowLogicalRect = Rect<WindowLogicalSpace>;

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
pub type ScreenNormalizedUnit = SpaceUnit<ScreenNormalizedSpace>;
pub type ScreenNormalizedPoint = SpacePoint<ScreenNormalizedSpace>;
pub type ScreenNormalizedVector = SpaceVector<ScreenNormalizedSpace>;

/// Space representing canvas space.
pub struct CanvasSpace;
pub type CanvasUnit = SpaceUnit<CanvasSpace>;
pub type CanvasPoint = SpacePoint<CanvasSpace>;
pub type CanvasVector = SpaceVector<CanvasSpace>;

pub trait SpaceUnderlyingConversion<U> {
  fn from_underlying(u: U) -> Self;
  fn into_underlying(self) -> U;
}

impl SpaceUnderlyingConversion<winit::dpi::PhysicalPosition<f32>> for WindowPhysicalPoint {
  fn from_underlying(u: winit::dpi::PhysicalPosition<f32>) -> Self {
    Self::new(u.x.into(), u.y.into())
  }
  fn into_underlying(self) -> winit::dpi::PhysicalPosition<f32> {
    winit::dpi::PhysicalPosition::new(self.x.into(), self.y.into())
  }
}

impl SpaceUnderlyingConversion<winit::dpi::PhysicalPosition<f64>> for WindowPhysicalPoint {
  fn from_underlying(u: winit::dpi::PhysicalPosition<f64>) -> Self {
    Self::new((u.x as f32).into(), (u.y as f32).into())
  }
  fn into_underlying(self) -> winit::dpi::PhysicalPosition<f64> {
    winit::dpi::PhysicalPosition::new(self.x.0 as f64, self.y.0 as f64)
  }
}

pub trait WindowLogicalPointExt {
  fn from_physical(point: WindowPhysicalPoint, scale_factor: f32) -> Self;
  fn from_screen(point: ScreenPixelPoint, camera: &Camera) -> Self;
}

impl WindowLogicalPointExt for WindowLogicalPoint {
  fn from_physical(point: WindowPhysicalPoint, scale_factor: f32) -> Self {
    let scale_factor = scale_factor.into();
    Self::new(
      (point.x / scale_factor).cast(),
      (point.y / scale_factor).cast(),
    )
  }

  fn from_screen(point: ScreenPixelPoint, camera: &Camera) -> Self {
    let screen_min = camera.screen_rect().min.coords.cast();
    let point = point.cast();
    point + screen_min
  }
}

impl SpaceUnderlyingConversion<egui::Pos2> for WindowLogicalPoint {
  fn from_underlying(u: egui::Pos2) -> Self {
    Self::new(u.x.into(), u.y.into())
  }

  fn into_underlying(self) -> egui::Pos2 {
    egui::Pos2::new(self.x.into(), self.y.into())
  }
}
impl SpaceUnderlyingConversion<egui::Rect> for WindowLogicalRect {
  fn from_underlying(u: egui::Rect) -> Self {
    Self::new(
      WindowLogicalPoint::from_underlying(u.min),
      WindowLogicalPoint::from_underlying(u.max),
    )
  }

  fn into_underlying(self) -> egui::Rect {
    egui::Rect::from_min_max(self.min.into_underlying(), self.max.into_underlying())
  }
}

pub trait ScreenPixelPointExt
where
  Self: Sized,
{
  fn from_window_logical(point: WindowLogicalPoint, camera: &Camera) -> Self;
  fn try_from_window_logical(point: WindowLogicalPoint, camera: &Camera) -> Option<Self>;
  fn from_canvas(point: CanvasPoint, camera: &Camera) -> Self;
}
impl ScreenPixelPointExt for ScreenPixelPoint {
  fn from_window_logical(point: WindowLogicalPoint, camera: &Camera) -> Self {
    let screen_min = camera.screen_rect().min.coords.cast();
    let point = point.cast();
    point - screen_min
  }

  fn try_from_window_logical(point: WindowLogicalPoint, camera: &Camera) -> Option<Self> {
    match camera.screen_rect().contains(point) {
      true => Some(Self::from_window_logical(point, camera)),
      false => None,
    }
  }

  fn from_canvas(canvas_point: CanvasPoint, camera: &Camera) -> Self {
    let canvas_point = canvas_point.cast();
    let view_point = camera.view_transform().transform_point(&canvas_point);
    let normalized_screen_point = camera.projection().transform_point(&view_point);
    let screen_point = camera
      .screen_transform()
      .transform_point(&normalized_screen_point);
    screen_point.cast()
  }
}

pub trait ScreenNormalizedVectorExt {
  fn from_pixel(vector: ScreenPixelVector, camera: &Camera) -> Self;
}
impl ScreenNormalizedVectorExt for ScreenNormalizedVector {
  fn from_pixel(vector: ScreenPixelVector, camera: &Camera) -> Self {
    let screen_size = camera.screen_rect().size();
    Self::new(
      (vector.x / screen_size.x.cast()).cast(),
      (vector.y / screen_size.y.cast()).cast(),
    )
  }
}

pub trait CanvasPointExt {
  fn from_screen(screen_point: ScreenPixelPoint, camera: &Camera) -> Self;
}
impl CanvasPointExt for CanvasPoint {
  fn from_screen(screen_point: ScreenPixelPoint, camera: &Camera) -> Self {
    let screen_point = screen_point.cast();
    let normalized_screen_point = camera
      .screen_transform()
      .inverse_transform_point(&screen_point);
    let view_point = camera
      .projection()
      .try_inverse_transform_point(&normalized_screen_point)
      .unwrap();
    let canvas_point = camera.view_transform().inverse_transform_point(&view_point);
    canvas_point.cast()
  }
}

pub trait CanvasVectorExt {
  fn from_screen(screen_vector: ScreenPixelVector, camera: &Camera) -> Self;
}
impl CanvasVectorExt for CanvasVector {
  fn from_screen(screen_vector: ScreenPixelVector, camera: &Camera) -> Self {
    let screen_vector = screen_vector.cast();
    let normalized_screen_vector = camera
      .screen_transform()
      .inverse_transform_vector(&screen_vector);
    let view_vector = camera.projection().try_inverse().unwrap() * normalized_screen_vector;
    let canvas_vector = camera
      .view_transform()
      .inverse_transform_vector(&view_vector);
    canvas_vector.cast()
  }
}
