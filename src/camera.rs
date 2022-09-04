use super::spaces::*;

#[derive(Debug, Clone)]
pub struct Camera {
  pub position: CanvasPoint,
  pub angle: f32,
  pub zoom: f32,
  pub viewport: egui::Rect,
}

impl Camera {
  pub fn init() -> Self {
    let position = CanvasPoint::origin();
    let angle = 0.0;
    let scale = 1.0;
    let screen = egui::Rect::NAN;

    Self {
      position,
      angle,
      zoom: scale,

      viewport: screen,
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
