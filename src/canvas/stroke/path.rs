use crate::{
  canvas::Camera,
  util::space::{CanvasPoint, CanvasPointExt},
};

use super::sample::SampledStroke;

use lyon::path::Path;

pub const DEFAULT_STROKE_WIDTH: f32 = 1.0;

pub struct PathStroke(pub Path);
impl Default for PathStroke {
  fn default() -> Self {
    Self(Path::new(1))
  }
}

impl PathStroke {
  pub fn new(sampled_stroke: &SampledStroke, camera: &Camera) -> Self {
    let mut points = sampled_stroke.samples.iter().map(|s| {
      let pos = CanvasPoint::from_screen(s.pos, camera);
      let pos = lyon::geom::Point::new(pos.x.0, pos.y.0);
      let force = s.force.unwrap_or(DEFAULT_STROKE_WIDTH);
      (pos, force)
    });

    let mut builder = Path::builder_with_attributes(1);
    let first_point = match points.next() {
      Some(s) => s,
      None => return Self::default(),
    };
    builder.begin(first_point.0, &[first_point.1]);
    for point in points {
      builder.line_to(point.0, &[point.1]);
    }
    builder.end(false);
    Self(builder.build())
  }
}
