use crate::{
  ui::{Renderer as UiRenderer, UiRenderTarget},
  util::space::*,
};

use euclid::Transform2D;

/// `CanvasPortal` is the portal between `CanvasSpace` and `WindowLogicalSpace`.
/// There is also intermediate `PortalSpace`.
///
/// The transformations between `CanvasSpace` and `PortalSpace` are affine (linear + translation).
/// They allow for translating, rotating and scaling.
///
/// The transformation between `WindowLogicalSpace` and `PortalSpace` are defined by the ui box.
///
/// The `CanvasPortal` (this portion of the canvas) is what we render.
pub struct CanvasPortal {
  /// The properties of the portal as seen from the canvas.
  pub position_canvas: CanvasPoint,
  /// The properties of the portal as seen from the canvas.
  pub rotation_canvas: f32,
  /// The properties of the portal as seen from the canvas.
  pub scale_canvas: f32,

  // The properties of the portal as seen from the window.
  window_box: WindowLogicalBox,

  /// Render target of the portal.
  render_target: UiRenderTarget,
}

impl CanvasPortal {
  pub fn init(device: &wgpu::Device, ui_renderer: &mut UiRenderer) -> Self {
    let position_canvas = CanvasPoint::default();
    let rotation_canvas = 0.0;
    let scale_canvas = 1.0;

    let window_box = WindowLogicalBox::default();
    let render_target = UiRenderTarget::new(device, ui_renderer);

    Self {
      position_canvas,
      rotation_canvas,
      scale_canvas,
      window_box,
      render_target,
    }
  }

  pub fn canvas_to_portal(&self) -> Transform2D<f32, CanvasSpace, PortalSpace> {
    let translation = -self.position_canvas.to_vector();
    let rotation = -self.rotation_canvas;
    let scale = 1.0 / self.scale_canvas;
    Transform2D::translation(translation.x, translation.y)
      .then_rotate(euclid::Angle::radians(rotation))
      .then_scale(scale, scale)
      .then_scale(
        1.0 / self.window_box.size().width,
        1.0 / self.window_box.size().height,
      )
  }

  pub fn portal_to_canvas(&self) -> Transform2D<f32, PortalSpace, CanvasSpace> {
    let translation = self.position_canvas.to_vector();
    Transform2D::scale(self.scale_canvas, self.scale_canvas)
      .then_scale(self.window_box.size().width, self.window_box.size().height)
      .then_rotate(euclid::Angle::radians(self.rotation_canvas))
      .then_translate(translation)
  }

  pub fn window_to_portal(&self) -> Transform2D<f32, WindowLogicalSpace, PortalSpace> {
    let scale = self.window_box.size();
    let translation = -self.window_box.center().to_vector();
    Transform2D::translation(translation.x, translation.y)
      .then_scale(1.0 / scale.width, 1.0 / scale.height)
  }

  pub fn portal_to_window(&self) -> Transform2D<f32, PortalSpace, WindowLogicalSpace> {
    let scale = self.window_box.size();
    let translation = self.window_box.center().to_vector();
    Transform2D::scale(scale.width, scale.height).then_translate(translation)
  }

  pub fn is_window_point_in_portal(&self, point: WindowLogicalPoint) -> bool {
    self.window_box.contains(point)
  }

  pub fn rotate_with_center(&mut self, rotation: f32, point: PortalPoint) {
    let mut tracker = PortalPoint::origin();
    let vector_to_point = point.to_vector();
    let transform = Transform2D::translation(-vector_to_point.x, -vector_to_point.y)
      .then_rotate(euclid::Angle::radians(rotation))
      .then_translate(vector_to_point);
    tracker = transform.transform_point(tracker);
    let shift = tracker - PortalPoint::origin();
    let shift = CanvasVector::from_portal(shift, self);

    self.position_canvas += shift;
    self.rotation_canvas = (self.rotation_canvas + rotation).rem_euclid(std::f32::consts::TAU);
  }

  pub fn scale_with_center(&mut self, scale: f32, point: PortalPoint) {
    let mut tracker = PortalPoint::origin();
    let vector_to_point = point.to_vector();
    let transform = Transform2D::translation(-vector_to_point.x, -vector_to_point.y)
      .then_scale(scale, scale)
      .then_translate(vector_to_point);
    tracker = transform.transform_point(tracker);
    let shift = tracker - PortalPoint::origin();
    let shift = CanvasVector::from_portal(shift, self);

    self.position_canvas += shift;
    self.scale_canvas *= scale;
  }
}

impl CanvasPortal {
  pub fn render_target(&self) -> &wgpu::TextureView {
    self.render_target.view()
  }

  pub fn ui(&mut self, ctx: &egui::Context, device: &wgpu::Device, ui_renderer: &mut UiRenderer) {
    egui::CentralPanel::default().show(ctx, |ui| {
      let response = self
        .render_target
        .ui(ui, device, ui_renderer, ui.available_size());

      self.window_box = WindowLogicalBox::from_underlying(response.rect);
    });
  }
}

// TODO: use this
//#[rustfmt::skip]
//pub const OPENGL_TO_WGPU_MATRIX: euclid::Matrix2D = cgmath::Matrix4::new(
//  1.0, 0.0, 0.0, 0.0,
//  0.0, 1.0, 0.0, 0.0,
//  0.0, 0.0, 0.5, 0.0,
//  0.0, 0.0, 0.5, 1.0,
//);
