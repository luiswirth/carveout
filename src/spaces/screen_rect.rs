use crate::math::Rect;

#[derive(Default)]
pub struct ScreenRect {
  pub(super) window_physical: Rect,
  pub(super) window_logical: Rect,
  pub(super) screen_physical: Rect,
  pub(super) screen_logical: Rect,
  pub(super) screen_norm: Rect,
  pub(super) canvas_view: Rect,
  pub(super) canvas: Rect,
}
impl std::fmt::Debug for ScreenRect {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ScreenRect")
      .field("window_logical", &self.window_logical)
      .finish()
  }
}

#[allow(dead_code)]
impl ScreenRect {
  pub fn aspect_ratio_xy(&self) -> f32 {
    self.window_logical.aspect_ratio_xy()
  }
  pub fn aspect_ratio_yx(&self) -> f32 {
    self.window_logical.aspect_ratio_yx()
  }
  pub fn size_norm_w(&self) -> na::Vector2<f32> {
    self.window_logical.size_norm_w()
  }

  pub fn window_physical(&self) -> Rect {
    self.window_physical
  }

  pub fn window_logical(&self) -> Rect {
    self.window_logical
  }

  pub fn screen_physical(&self) -> Rect {
    self.screen_physical
  }

  pub fn screen_logical(&self) -> Rect {
    self.screen_logical
  }

  pub fn screen_norm(&self) -> Rect {
    self.screen_norm
  }

  pub fn canvas_view(&self) -> Rect {
    self.canvas_view
  }

  pub fn canvas(&self) -> Rect {
    self.canvas
  }
}
