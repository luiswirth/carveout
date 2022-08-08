use crate::canvas::CanvasPortal;

pub struct WindowPhysicalSpace;
pub type WindowPhysicalPoint = euclid::Point2D<f32, WindowPhysicalSpace>;

pub struct WindowLogicalSpace;
pub type WindowLogicalPoint = euclid::Point2D<f32, WindowLogicalSpace>;
pub type WindowLogicalBox = euclid::Box2D<f32, WindowLogicalSpace>;

#[allow(dead_code)]
pub type UiPoint = WindowLogicalPoint;

pub struct PortalSpace;
pub type PortalPoint = euclid::Point2D<f32, PortalSpace>;
pub type PortalVector = euclid::Vector2D<f32, PortalSpace>;
pub type PortalLength = euclid::Length<f32, PortalSpace>;

pub struct CanvasSpace;
pub type CanvasPoint = euclid::Point2D<f32, CanvasSpace>;
pub type CanvasVector = euclid::Vector2D<f32, CanvasSpace>;

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
  fn from_portal(point: PortalPoint, portal: &CanvasPortal) -> Self;
}

impl WindowLogicalPointExt for WindowLogicalPoint {
  fn from_physical(point: WindowPhysicalPoint, scale_factor: f32) -> Self {
    Self::new(point.x / scale_factor, point.y / scale_factor)
  }

  fn from_portal(point: PortalPoint, portal: &CanvasPortal) -> Self {
    portal.portal_to_window().transform_point(point)
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

pub trait PortalPointExt
where
  Self: Sized,
{
  fn from_window_logical(point: WindowLogicalPoint, portal: &CanvasPortal) -> Self;
  fn try_from_window_logical(point: WindowLogicalPoint, portal: &CanvasPortal) -> Option<Self>;
  fn from_canvas(point: CanvasPoint, portal: &CanvasPortal) -> Self;
}
impl PortalPointExt for PortalPoint {
  fn from_window_logical(point: WindowLogicalPoint, portal: &CanvasPortal) -> Self {
    portal.window_to_portal().transform_point(point)
  }

  fn try_from_window_logical(point: WindowLogicalPoint, portal: &CanvasPortal) -> Option<Self> {
    match portal.is_window_point_in_portal(point) {
      true => Some(Self::from_window_logical(point, portal)),
      false => None,
    }
  }

  fn from_canvas(point: CanvasPoint, portal: &CanvasPortal) -> Self {
    portal.canvas_to_portal().transform_point(point)
  }
}

pub trait CanvasPointExt {
  fn from_portal(point: PortalPoint, portal: &CanvasPortal) -> Self;
}
impl CanvasPointExt for CanvasPoint {
  fn from_portal(point: PortalPoint, portal: &CanvasPortal) -> Self {
    portal.portal_to_canvas().transform_point(point)
  }
}

pub trait CanvasVectorExt {
  fn from_portal(vector: PortalVector, portal: &CanvasPortal) -> Self;
}
impl CanvasVectorExt for CanvasVector {
  fn from_portal(vector: PortalVector, portal: &CanvasPortal) -> Self {
    portal.portal_to_canvas().transform_vector(vector)
  }
}
