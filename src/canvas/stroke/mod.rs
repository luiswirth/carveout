mod render;
mod tessellate;

use self::{render::StrokeRenderer, tessellate::StrokeTessellator};
use super::{space::*, CameraWithScreen};

use lyon::path::Path;
use palette::LinSrgb;

pub struct StrokeManager {
  tessellator: StrokeTessellator,
  renderer: StrokeRenderer,
}

impl StrokeManager {
  pub fn init(device: &wgpu::Device) -> Self {
    let tessellator = StrokeTessellator::init();
    let renderer = StrokeRenderer::init(device);

    Self {
      tessellator,
      renderer,
    }
  }

  pub fn render<'a>(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    camera_screen: &CameraWithScreen,
    finished_strokes: impl Iterator<Item = &'a mut Stroke>,
    mut ongoing_stroke: Option<&'a mut Stroke>,
  ) {
    if let Some(ref mut s) = ongoing_stroke {
      s.path = Some(PathStroke::new(&s.sampled, camera_screen));
      s.tessellated = Some(
        self
          .tessellator
          .tessellate(s.path.as_ref().unwrap(), &s.shared_info),
      );
      let vertices = s
        .tessellated
        .as_ref()
        .unwrap()
        .0
        .vertices
        .iter()
        .map(|v| v.position.into())
        .collect();
      let indices: Vec<_> = s
        .tessellated
        .as_ref()
        .unwrap()
        .0
        .indices
        .clone()
        .chunks(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();
      if !indices.is_empty() {
        s.parry = Some(parry2d::shape::TriMesh::new(vertices, indices));
      }
    }

    let strokes = finished_strokes
      .chain(ongoing_stroke)
      .filter_map(|s| s.tessellated.as_mut());
    self
      .renderer
      .render(device, queue, encoder, camera_screen, strokes);
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrokeId(pub uuid::Uuid);
impl Default for StrokeId {
  fn default() -> Self {
    Self(uuid::Uuid::new_v4())
  }
}
impl StrokeId {
  pub fn nil() -> StrokeId {
    Self(uuid::Uuid::nil())
  }
}

#[derive(Default)]
pub struct Stroke {
  pub id: StrokeId,
  pub sampled: SampledStroke,
  pub path: Option<PathStroke>,
  pub tessellated: Option<TessallatedStroke>,
  pub parry: Option<parry2d::shape::TriMesh>,

  pub shared_info: SharedStrokeInfo,
}
impl Stroke {
  pub fn new(color: LinSrgb, width: f32) -> Self {
    Self {
      shared_info: SharedStrokeInfo { width, color },
      ..Default::default()
    }
  }
}

#[derive(Default)]
pub struct SampledStroke {
  pub samples: Vec<InteractionSample>,
}

pub struct InteractionSample {
  pub pos: ScreenPixelPoint,
}

#[derive(Default)]
pub struct TessallatedStroke(pub crate::gfx::tessellate::TessellationStore<render::Vertex>);

pub struct SharedStrokeInfo {
  pub width: f32,
  pub color: LinSrgb,
}

impl Default for SharedStrokeInfo {
  fn default() -> Self {
    Self {
      color: palette::named::WHITE.into_format().into_linear(),
      width: 1.0,
    }
  }
}

pub const DEFAULT_STROKE_WIDTH: f32 = 1.0;

pub struct PathStroke(pub Path);
impl Default for PathStroke {
  fn default() -> Self {
    Self(Path::new(1))
  }
}

impl PathStroke {
  pub fn new(sampled_stroke: &SampledStroke, camera_screen: &CameraWithScreen) -> Self {
    let mut points = sampled_stroke.samples.iter().map(|s| {
      let pos = CanvasPoint::from_screen(s.pos, camera_screen);
      lyon::geom::Point::new(pos.x.0, pos.y.0)
    });

    let mut builder = Path::builder_with_attributes(1);
    let first_point = match points.next() {
      Some(s) => s,
      None => return Self::default(),
    };
    builder.begin(first_point, &[DEFAULT_STROKE_WIDTH]);
    for point in points {
      builder.line_to(point, &[DEFAULT_STROKE_WIDTH]);
    }
    builder.end(false);
    Self(builder.build())
  }
}
