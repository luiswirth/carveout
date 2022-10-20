use super::render::{StrokeMeshCpu, StrokeVertex};

use crate::{gfx::mesh::Tessellation, stroke::Stroke, util};

use lyon::{
  lyon_tessellation::{BuffersBuilder, StrokeVertex as LyonStrokeVertex},
  path::{LineCap, Path},
  tessellation::{StrokeOptions, StrokeTessellator as LyonStrokeTessellator},
};

const DEFAULT_STROKE_WIDTH: f32 = 0.005;
const STROKE_WIDTH_ATTRIBUTE: lyon::path::AttributeIndex = 0;

#[derive(Default)]
pub struct StrokeTessellator {
  tessellator: LyonStrokeTessellator,
}

impl StrokeTessellator {
  pub fn tessellate(&mut self, stroke: &Stroke) -> StrokeMeshCpu {
    let mut points = stroke
      .points()
      .iter()
      .map(|p| lyon::geom::Point::new(p.x, p.y));
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

    let mut mesh = Tessellation::new();
    self
      .tessellator
      .tessellate_path(
        &path,
        &options,
        &mut BuffersBuilder::new(&mut mesh, |mut vertex: LyonStrokeVertex| StrokeVertex {
          position: vertex.position_on_path().to_array(),
          normal: vertex.normal().to_array(),
          stroke_width: vertex.interpolated_attributes()[0] * stroke.width_multiplier(),
          color: util::tuple2array4(stroke.color().into_components()),
        }),
      )
      .unwrap();
    StrokeMeshCpu::from_tessellation(mesh)
  }
}
