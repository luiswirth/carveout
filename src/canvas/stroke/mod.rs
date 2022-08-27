mod render;
mod tessellate;

use self::{render::StrokeRenderer, tessellate::StrokeTessellator};
use super::{content::StrokeId, space::*, CameraWithScreen};
use crate::gfx::tessellate::TessellationStore;

use palette::LinSrgb;
use std::collections::HashMap;

pub struct StrokeManager {
  data: StrokeData,
  tessellator: StrokeTessellator,
  renderer: StrokeRenderer,
}

impl StrokeManager {
  pub fn init(device: &wgpu::Device) -> Self {
    let data = StrokeData::default();
    let tessellator = StrokeTessellator::init();
    let renderer = StrokeRenderer::init(device);

    Self {
      data,
      tessellator,
      renderer,
    }
  }

  pub fn data(&self) -> &StrokeData {
    &self.data
  }

  pub fn clear_strokes(&mut self) {
    self.data = StrokeData::default();
  }

  pub fn update_strokes<'a>(&mut self, strokes: impl Iterator<Item = (StrokeId, &'a Stroke)>) {
    for (id, stroke) in strokes {
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

      self.data.tessellations.insert(id, tessellation);
      self.data.parry_meshes.insert(id, trimesh);
    }
  }

  pub fn render(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    camera_screen: &CameraWithScreen,
  ) {
    let stores = self.data.tessellations.values_mut();

    self
      .renderer
      .render(device, queue, encoder, camera_screen, stores);
  }
}

#[derive(Default)]
pub struct StrokeData {
  pub tessellations: HashMap<StrokeId, TessellationStore<render::Vertex>>,
  pub parry_meshes: HashMap<StrokeId, parry2d::shape::TriMesh>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Stroke {
  /// at least two points
  points: Vec<CanvasPoint>,
  width_multiplier: f32,
  color: LinSrgb,
}
impl Stroke {
  pub fn new(points: Vec<CanvasPoint>, color: LinSrgb, width_multiplier: f32) -> Self {
    assert!(points.len() >= 2);
    Self {
      points,
      color,
      width_multiplier,
    }
  }

  pub fn add_point(&mut self, point: CanvasPoint) {
    self.points.push(point);
  }
}
