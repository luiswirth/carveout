#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Rect {
  pub extents_half: na::Vector2<f32>,
  pub center: na::Point2<f32>,
  pub angle: f32,
}
impl Eq for Rect {}

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

  /// requires that `vs` form a valid rect.
  pub fn from_vertices(vs: [na::Point2<f32>; 4]) -> Self {
    let extents_half = na::vector!(
      (vs[3] - vs[0]).magnitude() / 2.0,
      (vs[1] - vs[0]).magnitude() / 2.0
    );
    let center = vs
      .iter()
      .copied()
      .map(|v| v.coords)
      .sum::<na::Vector2<f32>>()
      .scale(1.0 / 4.0)
      .into();

    let left_to_right_top = (vs[3] - vs[0]).normalize();
    let angle = left_to_right_top.y.atan2(left_to_right_top.x);

    Rect {
      extents_half,
      center,
      angle,
    }
  }

  pub fn vertices(&self) -> [na::Point2<f32>; 4] {
    let w = na::Vector2::new(self.extents_half.x, 0.0);
    let h = na::Vector2::new(0.0, self.extents_half.y);
    let mut vecs = [-w - h, -w + h, w + h, w - h];
    let rotation = self.rotation();
    vecs = vecs.map(|vec| rotation * vec);
    vecs.map(|vec| self.center + vec)
  }
  pub fn rotation(&self) -> na::Rotation2<f32> {
    na::Rotation2::new(self.angle)
  }
  pub fn min(&self) -> na::Point2<f32> {
    self.rotation() * (self.center - self.extents_half)
  }
  pub fn max(&self) -> na::Point2<f32> {
    self.rotation() * (self.center + self.extents_half)
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
}
