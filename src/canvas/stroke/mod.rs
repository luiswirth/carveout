mod render;
mod tessellate;

use self::{render::StrokeRenderer, tessellate::StrokeTessellator};
use super::{
  content::{
    access::{ContentAccess, StrokeDelta},
    StrokeId,
  },
  space::*,
  CameraWithScreen,
};
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

  pub fn update_strokes(&mut self, content: ContentAccess, stroke_delta: &StrokeDelta) {
    let need_update = stroke_delta
      .added
      .iter()
      .chain(stroke_delta.modified.iter())
      .copied();

    for stroke_id in need_update {
      let stroke = content.stroke(stroke_id);
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

      self.data.tessellations.insert(stroke_id, tessellation);
      self.data.parry_meshes.insert(stroke_id, trimesh);
    }

    for stroke_id in stroke_delta.removed.iter() {
      self.data.tessellations.remove(stroke_id);
      self.data.parry_meshes.remove(stroke_id);
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
