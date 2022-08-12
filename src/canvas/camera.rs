use crate::{
  ui::{Renderer as UiRenderer, UiRenderTarget},
  util::space::*,
};

pub struct Camera {
  pub position: CanvasPoint,
  pub angle: f32,
  pub scale: f32,

  screen_rect: WindowLogicalRect,
  render_target: UiRenderTarget,
}

impl Camera {
  pub fn init(device: &wgpu::Device, ui_renderer: &mut UiRenderer) -> Self {
    let position = CanvasPoint::origin();
    let angle = 0.0;
    let scale = 1.0;

    let screen_rect = WindowLogicalRect::default();
    let render_target = UiRenderTarget::new(device, ui_renderer);

    Self {
      position,
      angle,
      scale,
      screen_rect,
      render_target,
    }
  }

  // view transform (mVp)
  // canvas space -> view space
  pub fn view_transform(&self) -> na::IsometryMatrix2<f32> {
    let rotation = na::Rotation2::new(-self.angle);
    let translation = na::Translation2::from(-self.position.cast());
    translation * rotation
  }

  // projection (mvP)
  // view space -> normalized screen space
  pub fn projection(&self) -> na::Scale2<f32> {
    let camera_scale = self.scale;
    let camera_scale = na::Scale2::new(camera_scale, camera_scale);
    let screen_scale = self.screen_rect.size().cast::<f32>().map(|e| 2.0 / e);
    let screen_scale = na::Scale2::from(screen_scale);
    screen_scale * camera_scale
  }

  // viewport transform
  // normalized screen space -> screen space
  pub fn screen_transform(&self) -> na::Affine2<f32> {
    let translation = na::Translation2::new(1.0, 1.0);
    let translation: na::Affine2<f32> = na::convert(translation);
    let scale = self.screen_rect.size().cast() / 2.0;
    let scale = na::Scale2::from(scale);
    let scale: na::Affine2<f32> = na::convert(scale);

    scale * translation
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

  pub fn screen_rect(&self) -> &WindowLogicalRect {
    &self.screen_rect
  }
}

impl Camera {
  pub fn render_target(&self) -> &wgpu::TextureView {
    self.render_target.view()
  }

  pub fn ui(&mut self, ctx: &egui::Context, device: &wgpu::Device, ui_renderer: &mut UiRenderer) {
    egui::CentralPanel::default().show(ctx, |ui| {
      let response = self
        .render_target
        .ui(ui, device, ui_renderer, ui.available_size());

      self.screen_rect = WindowLogicalRect::from_underlying(response.rect);
    });
  }
}
