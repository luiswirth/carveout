use crate::{
  content::{
    access::{ContentAccess, StrokeDelta},
    StrokeId,
  },
  gfx::stroke::{StrokeMeshGpu, StrokeTessellator},
  spaces::*,
};

use palette::LinSrgb;
use std::collections::HashMap;

#[derive(Default)]
pub struct StrokeManager {
  data: StrokeData,
  tessellator: StrokeTessellator,
}

impl StrokeManager {
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
}

// TODO: consider using a BTreeMap instead of a HashMap
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

  pub fn points(&self) -> &[CanvasPoint] {
    &self.points
  }

  pub fn width_multiplier(&self) -> f32 {
    self.width_multiplier
  }

  pub fn color(&self) -> LinSrgb {
    self.color
  }
}
