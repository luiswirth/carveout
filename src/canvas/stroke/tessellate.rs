use super::{
  render::{Vertex, VertexConstructor},
  Stroke,
};
use crate::{
  gfx::tessellate::{TessellationStore, TessellationStoreBuilder},
  util,
};

use lyon::{
  path::{LineCap, Path},
  tessellation::{StrokeOptions, StrokeTessellator as LyonStrokeTessellator},
};

pub struct StrokeTessellator {
  tessellator: LyonStrokeTessellator,
}

const DEFAULT_STROKE_WIDTH: f32 = 1.0;
const STROKE_WIDTH_ATTRIBUTE: lyon::path::AttributeIndex = 0;

impl StrokeTessellator {
  pub fn init() -> Self {
    let tessellator = LyonStrokeTessellator::new();
    Self { tessellator }
  }

  #[allow(clippy::suspicious_map)]
  pub fn tessellate(&mut self, stroke: &Stroke) -> TessellationStore<Vertex> {
    // build path
    let mut points = stroke
      .points
      .iter()
      .map(|p| lyon::geom::Point::new(p.x.0, p.y.0));
    let mut builder = Path::builder_with_attributes(1);
    let first_point = points.next().unwrap();
    builder.begin(first_point, &[DEFAULT_STROKE_WIDTH]);
    for point in points {
      builder.line_to(point, &[DEFAULT_STROKE_WIDTH]);
    }
    builder.end(false);
    let path = builder.build();

    let options = StrokeOptions::default()
      .with_tolerance(0.001)
      .with_line_cap(LineCap::Round)
      .with_variable_line_width(STROKE_WIDTH_ATTRIBUTE);

    let mut store = TessellationStore::default();
    self
      .tessellator
      .tessellate_path(
        &path,
        &options,
        &mut TessellationStoreBuilder::new(
          &mut store,
          VertexConstructor {
            width_multiplier: stroke.width_multiplier,
            color: stroke.color.into(),
          },
        ),
      )
      .expect("tessellation failed");

    store
  }
}

impl lyon::lyon_tessellation::StrokeVertexConstructor<Vertex> for VertexConstructor {
  fn new_vertex(&mut self, mut vertex: lyon::tessellation::StrokeVertex<'_, '_>) -> Vertex {
    Vertex {
      position: vertex.position_on_path().to_array(),
      normal: vertex.normal().to_array(),
      stroke_width: vertex.interpolated_attributes()[0] * self.width_multiplier,
      color: util::tuple2array4(self.color.into_components()),
    }
  }
}
