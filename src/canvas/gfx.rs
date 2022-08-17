use std::{
  cell::{Ref, RefCell},
  rc::Rc,
};

use crate::ui::CanvasScreen;

use super::space::CanvasPoint;

pub struct CameraWithScreen {
  camera: Camera,
  screen: Rc<RefCell<CanvasScreen>>,
}

impl CameraWithScreen {
  pub fn init(screen: Rc<RefCell<CanvasScreen>>) -> Self {
    let camera = Camera::init();
    Self { camera, screen }
  }

  // view transform (mVp)
  // canvas space -> view space
  pub fn view_transform(&self) -> na::IsometryMatrix2<f32> {
    let rotation = na::Rotation2::new(-self.camera.angle);
    let translation = na::Translation2::from(-self.camera.position.cast());
    translation * rotation
  }

  // projection (mvP)
  // view space -> normalized screen space
  pub fn projection(&self) -> na::Scale2<f32> {
    let camera_scale = self.camera.scale;
    let camera_scale = na::Scale2::new(camera_scale, camera_scale);
    let screen_scale = self.screen_rect().size();
    let screen_scale = na::Scale2::new(2.0 / screen_scale.x, 2.0 / screen_scale.y);
    screen_scale * camera_scale
  }

  // viewport transform
  // normalized screen space -> screen space
  pub fn screen_transform(&self) -> na::Affine2<f32> {
    let translation = na::Translation2::new(1.0, 1.0);
    let translation: na::Affine2<f32> = na::convert(translation);
    let scale = self.screen_rect().size();
    let scale = na::Scale2::new(scale.x / 2.0, scale.y / 2.0);
    let scale: na::Affine2<f32> = na::convert(scale);

    scale * translation
  }

  pub fn render_target(&self) -> Ref<wgpu::TextureView> {
    Ref::map(self.screen.borrow(), |s| s.render_target())
  }

  pub fn screen_rect(&self) -> egui::Rect {
    self.screen.borrow().rect()
  }

  pub fn camera_mut(&mut self) -> &mut Camera {
    &mut self.camera
  }
}

pub struct Camera {
  pub position: CanvasPoint,
  pub angle: f32,
  pub scale: f32,
}

impl Camera {
  pub fn init() -> Self {
    let position = CanvasPoint::origin();
    let angle = 0.0;
    let scale = 1.0;

    Self {
      position,
      angle,
      scale,
    }
  }

  pub fn rotate_with_center(&mut self, angle: f32, _center: CanvasPoint) {
    // TODO
    //let translation: na::Transform2<f32> = na::convert(na::Translation2::from(center.coords));
    //let rotation: na::Transform2<f32> = na::convert(na::Rotation2::new(angle));
    //let translation_inv: na::Transform2<f32> = na::convert(na::Translation2::from(-center.coords));
    //let transformation: na::Transform2<CanvasUnit> =
    //  na::convert(translation_inv * rotation * translation);
    //self.position = transformation.transform_point(&self.position);

    self.angle = (self.angle + angle).rem_euclid(std::f32::consts::TAU);
  }

  pub fn scale_with_center(&mut self, scale: f32, _center: CanvasPoint) {
    // TODO
    //let translation: na::Transform2<f32> = na::convert(na::Translation2::from(center.coords));
    //let scaling: na::Transform2<f32> = na::convert(na::Scale2::new(scale, scale));
    //let translation_inv: na::Transform2<f32> = na::convert(na::Translation2::from(-center.coords));
    //let transformation: na::Transform2<CanvasUnit> =
    //  na::convert(translation_inv * scaling * translation);
    //self.position = transformation.transform_point(&self.position);

    self.scale *= scale;
  }
}
