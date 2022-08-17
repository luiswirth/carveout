use super::{
  render::{Vertex, VertexConstructor},
  PathStroke, SharedStrokeInfo, TessallatedStroke,
};
use crate::{gfx::tessellate::TessellationStoreBuilder, util};

use lyon::{
  path::LineCap,
  tessellation::{StrokeOptions, StrokeTessellator as LyonStrokeTessellator},
};

pub struct StrokeTessellator {
  tessellator: LyonStrokeTessellator,
}

impl StrokeTessellator {
  pub fn init() -> Self {
    let tessellator = LyonStrokeTessellator::new();
    Self { tessellator }
  }

  #[allow(clippy::suspicious_map)]
  pub fn tessellate(
    &mut self,
    path_stroke: &PathStroke,
    stroke_info: &SharedStrokeInfo,
  ) -> TessallatedStroke {
    const STROKE_WIDTH_ATTRIBUTE: lyon::path::AttributeIndex = 0;

    let options = StrokeOptions::default()
      .with_tolerance(0.001)
      .with_line_cap(LineCap::Round)
      .with_variable_line_width(STROKE_WIDTH_ATTRIBUTE);

    let mut tessellated_stroke = TessallatedStroke::default();

    self
      .tessellator
      .tessellate_path(
        &path_stroke.0,
        &options,
        &mut TessellationStoreBuilder::new(
          &mut tessellated_stroke.0,
          VertexConstructor {
            stroke_width: stroke_info.width,
            color: stroke_info.color.into(),
          },
        ),
      )
      .expect("tessellation failed");

    tessellated_stroke
  }
}

impl lyon::lyon_tessellation::StrokeVertexConstructor<Vertex> for VertexConstructor {
  fn new_vertex(&mut self, mut vertex: lyon::tessellation::StrokeVertex<'_, '_>) -> Vertex {
    Vertex {
      position: vertex.position_on_path().to_array(),
      normal: vertex.normal().to_array(),
      stroke_width: vertex.interpolated_attributes()[0] * self.stroke_width,
      color: util::tuple2array4(self.color.into_components()),
    }
  }
}
