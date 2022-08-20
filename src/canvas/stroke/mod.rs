mod render;
mod tessellate;

use self::{render::StrokeRenderer, tessellate::StrokeTessellator};
use super::{space::*, CameraWithScreen};
use crate::gfx::tessellate::TessellationStore;

use palette::LinSrgb;
use std::collections::HashMap;

pub struct StrokeManager {
  data: HashMap<StrokeId, StrokeData>,
  tessellator: StrokeTessellator,
  renderer: StrokeRenderer,
}

impl StrokeManager {
  pub fn init(device: &wgpu::Device) -> Self {
    let data = HashMap::new();
    let tessellator = StrokeTessellator::init();
    let renderer = StrokeRenderer::init(device);

    Self {
      data,
      tessellator,
      renderer,
    }
  }

  pub fn data(&self) -> &HashMap<StrokeId, StrokeData> {
    &self.data
  }

  pub fn clear_strokes(&mut self) {
    self.data.clear();
  }

  pub fn update_strokes<'a>(&mut self, strokes: impl IntoIterator<Item = &'a Stroke>) {
    for stroke in strokes {
      let tessellation = self.tessellator.tessellate(stroke);

      let vertices = tessellation
        .vertices
        .iter()
        .map(|v| v.position.into())
        .collect();
      let indices = tessellation
        .indices
        .chunks(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();

      let trimesh = parry2d::shape::TriMesh::new(vertices, indices);

      let data = StrokeData {
        tessellation,
        parry_mesh: trimesh,
      };
      self.data.insert(stroke.id, data);
    }
  }

  pub fn render(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    camera_screen: &CameraWithScreen,
  ) {
    let stores = self.data.values_mut().map(|d| &mut d.tessellation);

    self
      .renderer
      .render(device, queue, encoder, camera_screen, stores);
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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

pub struct StrokeData {
  pub tessellation: TessellationStore<render::Vertex>,
  pub parry_mesh: parry2d::shape::TriMesh,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Stroke {
  id: StrokeId,

  /// at least two points
  points: Vec<CanvasPoint>,
  width_multiplier: f32,
  color: LinSrgb,
}
impl Stroke {
  pub fn new(points: Vec<CanvasPoint>, color: LinSrgb, width_multiplier: f32) -> Self {
    assert!(points.len() >= 2);
    let id = StrokeId::default();
    Self {
      id,
      points,
      color,
      width_multiplier,
    }
  }

  pub fn id(&self) -> StrokeId {
    self.id
  }

  pub fn add_point(&mut self, point: CanvasPoint) {
    self.points.push(point);
  }
}
