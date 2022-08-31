mod render;
mod tessellate;

use self::{
  render::{StrokeMeshGpu, StrokeRenderer},
  tessellate::StrokeTessellator,
};
use super::{
  content::{
    access::{ContentAccess, StrokeDelta},
    StrokeId,
  },
  space::*,
  CameraWithScreen,
};

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

  pub fn update_strokes(
    &mut self,
    content: ContentAccess,
    stroke_delta: &StrokeDelta,
    device: &wgpu::Device,
  ) {
    let need_update = stroke_delta
      .added
      .iter()
      .chain(stroke_delta.modified.iter())
      .copied();

    for stroke_id in need_update {
      let stroke = content.stroke(stroke_id);
      let mesh = self.tessellator.tessellate(stroke);

      let vertices = mesh.vertices().iter().map(|v| v.position.into()).collect();
      let indices = mesh
        .indices()
        .chunks(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();
      let trimesh = parry2d::shape::TriMesh::new(vertices, indices);

      self
        .data
        .meshes
        .insert(stroke_id, StrokeMeshGpu::from_mesh_cpu(&mesh, device));
      self.data.parry_meshes.insert(stroke_id, trimesh);
    }

    for stroke_id in stroke_delta.removed.iter() {
      self.data.meshes.remove(stroke_id);
      self.data.parry_meshes.remove(stroke_id);
    }
  }

  pub fn render(
    &mut self,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    camera_screen: &CameraWithScreen,
  ) {
    let meshes = self.data.meshes.iter_mut().map(|(_, mesh)| mesh);
    self.renderer.render(queue, encoder, camera_screen, meshes);
  }
}

#[derive(Default)]
pub struct StrokeData {
  pub meshes: HashMap<StrokeId, StrokeMeshGpu>,
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
