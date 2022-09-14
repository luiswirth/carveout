#[derive(Debug, Copy, Clone, Default)]
pub struct Rect {
  pub extents_half: na::Vector2<f32>,
  pub center: na::Point2<f32>,
  pub angle: f32,
}

#[allow(dead_code)]
impl Rect {
  pub fn from_extents_half_center(extents_half: na::Vector2<f32>, center: na::Point2<f32>) -> Self {
    Self {
      extents_half,
      center,
      angle: 0.0,
    }
  }

  pub fn from_size_center(size: na::Vector2<f32>, center: na::Point2<f32>) -> Self {
    Self {
      extents_half: size.scale(0.5),
      center,
      angle: 0.0,
    }
  }

  pub fn from_size_min(size: na::Vector2<f32>, min: na::Point2<f32>) -> Self {
    Self {
      extents_half: size.scale(0.5),
      center: min + size.scale(0.5),
      angle: 0.0,
    }
  }

  pub fn min(&self) -> na::Point2<f32> {
    self.center - self.extents_half
  }
  pub fn max(&self) -> na::Point2<f32> {
    self.center + self.extents_half
  }
  pub fn size(&self) -> na::Vector2<f32> {
    self.extents_half.scale(2.0)
  }
  pub fn aspect_ratio_xy(&self) -> f32 {
    self.extents_half.x / self.extents_half.y
  }
  pub fn aspect_ratio_yx(&self) -> f32 {
    self.extents_half.y / self.extents_half.x
  }
  pub fn size_norm_w(&self) -> na::Vector2<f32> {
    let w = 1.0;
    let h = self.aspect_ratio_yx() * w;
    na::vector![w, h]
  }
  pub fn size_norm_h(&self) -> na::Vector2<f32> {
    let h = 1.0;
    let w = self.aspect_ratio_xy() * h;
    na::vector![w, h]
  }
  pub fn shape(&self) -> parry2d::shape::Cuboid {
    parry2d::shape::Cuboid::new(self.extents_half)
  }
  pub fn isometry(&self) -> na::Isometry2<f32> {
    na::Isometry2::new(self.center.coords, self.angle)
  }

  pub fn contains(&self, p: na::Point2<f32>) -> bool {
    let min = self.min();
    let max = self.max();
    min.x <= p.x && p.x <= max.x && min.y <= p.y && p.y <= max.y
  }
}
