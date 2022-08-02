use crate::canvas::CanvasViewport;

pub struct WindowPhysicalSpace;
pub type WindowPhysicalPoint = euclid::Point2D<f32, WindowPhysicalSpace>;

pub struct WindowLogicalSpace;
pub type WindowLogicalPoint = euclid::Point2D<f32, WindowLogicalSpace>;
pub type WindowLogicalBox = euclid::Box2D<f32, WindowLogicalSpace>;

#[allow(dead_code)]
pub type UiPoint = WindowLogicalPoint;

/// Center is (0, 0)
pub struct CanvasViewportSpace;
pub type CanvasViewportPoint = euclid::Point2D<f32, CanvasViewportSpace>;
pub type CanvasViewportLength = euclid::Length<f32, CanvasViewportSpace>;

/// Center is (0, 0)
pub struct CanvasSpace;
pub type CanvasPoint = euclid::Point2D<f32, CanvasSpace>;

pub trait SpaceUnderlyingConversion<U> {
  fn from_underlying(u: U) -> Self;
  fn into_underlying(self) -> U;
}

impl SpaceUnderlyingConversion<winit::dpi::PhysicalPosition<f32>> for WindowPhysicalPoint {
  fn from_underlying(u: winit::dpi::PhysicalPosition<f32>) -> Self {
    Self::new(u.x, u.y)
  }
  fn into_underlying(self) -> winit::dpi::PhysicalPosition<f32> {
    winit::dpi::PhysicalPosition::new(self.x, self.y)
  }
}

impl SpaceUnderlyingConversion<winit::dpi::PhysicalPosition<f64>> for WindowPhysicalPoint {
  fn from_underlying(u: winit::dpi::PhysicalPosition<f64>) -> Self {
    Self::new(u.x as f32, u.y as f32)
  }
  fn into_underlying(self) -> winit::dpi::PhysicalPosition<f64> {
    winit::dpi::PhysicalPosition::new(self.x as f64, self.y as f64)
  }
}

pub trait WindowLogicalPointExt {
  fn from_physical(point: WindowPhysicalPoint, scale_factor: f32) -> Self;
}

impl WindowLogicalPointExt for WindowLogicalPoint {
  fn from_physical(point: WindowPhysicalPoint, scale_factor: f32) -> Self {
    Self::new(point.x / scale_factor, point.y / scale_factor)
  }
}

impl SpaceUnderlyingConversion<egui::Pos2> for WindowLogicalPoint {
  fn from_underlying(u: egui::Pos2) -> Self {
    Self::new(u.x, u.y)
  }

  fn into_underlying(self) -> egui::Pos2 {
    egui::Pos2::new(self.x, self.y)
  }
}
impl SpaceUnderlyingConversion<egui::Rect> for WindowLogicalBox {
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

pub trait CanvasViewportPointExt
where
  Self: Sized,
{
  fn from_window_logical(
    window: WindowLogicalPoint,
    viewport_box: WindowLogicalBox,
  ) -> Option<Self>;
  fn from_canvas(point: CanvasPoint, viewport: &CanvasViewport) -> Self;
}
impl CanvasViewportPointExt for CanvasViewportPoint {
  fn from_window_logical(
    point: WindowLogicalPoint,
    viewport_box: WindowLogicalBox,
  ) -> Option<Self> {
    match viewport_box.contains(point) {
      true => {
        let viewport_size = viewport_box.size();
        let point = point - viewport_box.center();
        Some(CanvasViewportPoint::new(
          point.x / viewport_size.width,
          point.y / viewport_size.height,
        ))
      }
      false => None,
    }
  }

  fn from_canvas(point: CanvasPoint, viewport: &CanvasViewport) -> Self {
    viewport.transform.transform_point(point)
  }
}

pub trait CanvasPointExt {
  fn from_viewport(point: CanvasViewportPoint, viewport: &CanvasViewport) -> Self;
}
impl CanvasPointExt for CanvasPoint {
  fn from_viewport(point: CanvasViewportPoint, viewport: &CanvasViewport) -> Self {
    viewport
      .transform
      .inverse()
      .expect("must be invertible")
      .transform_point(point)
  }
}
