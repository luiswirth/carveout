mod camera;

pub use self::camera::Camera;

use crate::{input::InputManager, math::Rect, natrans};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Space {
  /// window physical pixels coordinate system.
  /// origin upper left corner
  /// units in physical pixels (square)
  /// same as winit physical
  WindowPhysical,
  /// window logical pixels coordinate system.
  /// origin upper left corner
  /// units in logical pixels (square)
  /// same as winit logical and egui
  WindowLogical,
  /// canvas screen physical pixels coordinate system.
  /// origin upper left corner
  /// units in physical pixels (square)
  ScreenPhysical,
  /// canvas screen logical pixels coordinate system.
  /// origin upper left corner
  /// units in logical pixels (square)
  ScreenLogical,
  /// canvas normalized coordinate system.
  /// origin center
  /// units normalized [-1,+1]x[-1,+1] (non square)
  /// same as gfx screen clip space / normalized device coordinates
  ScreenNorm,
  /// camera view coordinate system.
  /// origin camera position
  /// units canvas
  /// same as gfx view (square)
  View,
  /// canvas coordinate system
  /// origin center
  /// units initially same as screen norm
  Canvas,
}

/// holds data necessary for all relevant space transformations
#[derive(Debug, Default)]
pub struct SpaceManager {
  camera: Camera,
  screen_rect_window_logical: Rect,
  scale_factor: f32,
}

impl SpaceManager {
  pub fn update_camera_controller(&mut self, input_manager: &InputManager) {
    camera::controller::update(self, input_manager);
  }

  pub fn update_screen_rect(&mut self, new_egui_rect: egui::Rect) {
    let size_logical = mint::Vector2::from(new_egui_rect.size()).into();
    let center_window_logical = mint::Point2::from(new_egui_rect.center()).into();
    self.screen_rect_window_logical = Rect::from_size_center(size_logical, center_window_logical);
  }

  pub fn update_scale_factor(&mut self, scale_factor: f32) {
    self.scale_factor = scale_factor;
  }

  pub fn camera(&self) -> &Camera {
    &self.camera
  }

  pub fn camera_mut(&mut self) -> &mut Camera {
    &mut self.camera
  }

  pub fn screen_rect_window_logical(&self) -> Rect {
    self.screen_rect_window_logical
  }
}

#[allow(dead_code)]
impl SpaceManager {
  pub fn physical_to_logical(&self) -> na::Scale2<f32> {
    na::Scale2::new(1.0 / self.scale_factor, 1.0 / self.scale_factor)
  }
  pub fn logical_to_physical(&self) -> na::Scale2<f32> {
    na::Scale2::new(self.scale_factor, self.scale_factor)
  }

  pub fn window_to_screen_logical(&self) -> na::Translation2<f32> {
    na::Translation2::from(-self.screen_rect_window_logical.min().coords)
  }
  pub fn screen_to_window_logical(&self) -> na::Translation2<f32> {
    na::Translation2::from(self.screen_rect_window_logical.min().coords)
  }

  pub fn window_to_screen_physical(&self) -> na::Translation2<f32> {
    na::Translation2::from(-self.screen_rect_window_logical.min().coords * self.scale_factor)
  }
  pub fn screen_to_window_physical(&self) -> na::Translation2<f32> {
    na::Translation2::from(self.screen_rect_window_logical.min().coords * self.scale_factor)
  }

  pub fn screen_logical_to_norm(&self) -> na::Affine2<f32> {
    let size = self.screen_rect_window_logical.size();
    let scale = na::Scale2::new(2.0 / size.x, 2.0 / size.y);
    let translation = na::Translation2::new(-1.0, -1.0);

    let scale: na::Affine2<f32> = na::convert(scale);
    let translation: na::Affine2<f32> = na::convert(translation);
    translation * scale
  }
  // gfx: viewport transform
  pub fn screen_norm_to_logical(&self) -> na::Affine2<f32> {
    let translation = na::Translation2::new(1.0, 1.0);
    let size = self.screen_rect_window_logical.size();
    let scale = na::Scale2::new(size.x / 2.0, size.y / 2.0);

    let translation: na::Affine2<f32> = na::convert(translation);
    let scale: na::Affine2<f32> = na::convert(scale);
    scale * translation
  }

  pub fn screen_norm_to_canvas_view(&self) -> na::Scale2<f32> {
    let screen_aspect_scale = na::Scale2::from(self.screen_rect_window_logical.size_norm_w());
    let camera_zoom = na::Scale2::new(1.0 / self.camera.zoom, 1.0 / self.camera.zoom);
    camera_zoom * screen_aspect_scale
  }
  // gfx: projection (mvP)
  pub fn canvas_view_to_screen_norm(&self) -> na::Scale2<f32> {
    let camera_zoom = na::Scale2::new(self.camera.zoom, self.camera.zoom);
    let screen_aspect_scale = na::Scale2::from(
      self
        .screen_rect_window_logical
        .size_norm_w()
        .map(|e| 1.0 / e),
    );
    screen_aspect_scale * camera_zoom
  }

  pub fn view_to_canvas(&self) -> na::IsometryMatrix2<f32> {
    let rotation = na::Rotation2::new(self.camera.angle);
    let translation = na::Translation2::from(self.camera.position_canvas);
    translation * rotation
  }
  /// gfx: view transform (mVp)
  pub fn canvas_to_view(&self) -> na::IsometryMatrix2<f32> {
    let translation = na::Translation2::from(-self.camera.position_canvas);
    let rotation = na::Rotation2::new(-self.camera.angle);
    rotation * translation
  }
}

impl SpaceManager {
  pub fn transform_point(&self, point: na::Point2<f32>, src: Space, dst: Space) -> na::Point2<f32> {
    use Space::*;
    match [src, dst] {
      [ScreenLogical, Canvas] => {
        natrans!(self.view_to_canvas())
          * natrans!(self.screen_norm_to_canvas_view())
          * natrans!(self.screen_logical_to_norm())
          * point
      }
      [Canvas, ScreenLogical] => {
        natrans!(self.screen_norm_to_logical())
          * natrans!(self.canvas_view_to_screen_norm())
          * natrans!(self.canvas_to_view())
          * point
      }
      [WindowPhysical, ScreenLogical] => {
        natrans!(self.window_to_screen_logical()) * natrans!(self.physical_to_logical()) * point
      }
      [ScreenLogical, WindowLogical] => natrans!(self.screen_to_window_logical()) * point,
      _ => unimplemented!("`transform_point` from {src:?} to {dst:?} unimplemented.",),
    }
  }
  pub fn transform_vector(
    &self,
    vector: na::Vector2<f32>,
    src: Space,
    dst: Space,
  ) -> na::Vector2<f32> {
    use Space::*;
    match [src, dst] {
      [WindowLogical, WindowPhysical] => self.logical_to_physical() * vector,
      [ScreenLogical, Canvas] => {
        natrans!(self.view_to_canvas())
          * natrans!(self.screen_norm_to_canvas_view())
          * natrans!(self.screen_logical_to_norm())
          * vector
      }
      [ScreenNorm, Canvas] => {
        natrans!(self.view_to_canvas()) * natrans!(self.screen_norm_to_canvas_view()) * vector
      }
      [WindowPhysical, ScreenLogical] => {
        natrans!(self.physical_to_logical()) * natrans!(self.window_to_screen_logical()) * vector
      }
      [ScreenLogical, ScreenNorm] => self.screen_logical_to_norm() * vector,
      _ => unimplemented!("`transform_vector` from {src:?} to {dst:?} unimplemented.",),
    }
  }
  pub fn transform_rect(&self, rect: Rect, src: Space, dst: Space) -> Rect {
    use Space::*;
    match [src, dst] {
      [WindowLogical, WindowPhysical] => {
        let center = self.logical_to_physical() * rect.center;
        let extents_half = self.logical_to_physical() * rect.extents_half;
        Rect::from_extents_half_center(extents_half, center)
      }
      [WindowLogical, Canvas] => {
        let transformation = natrans!(self.view_to_canvas())
          * natrans!(self.screen_norm_to_canvas_view())
          * natrans!(self.screen_logical_to_norm())
          * natrans!(self.window_to_screen_logical());

        Rect::from_vertices(rect.vertices().map(|v| transformation * v))
      }
      _ => unimplemented!("`transform_rect` from {src:?} to {dst:?} unimplemented.",),
    }
  }
}
