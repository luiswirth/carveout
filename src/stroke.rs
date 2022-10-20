use crate::{
  content::{
    access::{ContentAccess, StrokeDelta},
    StrokeId,
  },
  gfx::stroke::{StrokeMeshGpu, StrokeTessellator},
};

use palette::{LinSrgb, LinSrgba};
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
  points_canvas: Vec<na::Point2<f32>>,
  width_multiplier: f32,
  color: palette::LinSrgba,
}
impl Stroke {
  pub fn new(
    points: Vec<na::Point2<f32>>,
    color: palette::LinSrgba,
    width_multiplier: f32,
  ) -> Self {
    assert!(points.len() >= 2);
    Self {
      points_canvas: points,
      color,
      width_multiplier,
    }
  }

  pub fn add_point(&mut self, point: na::Point2<f32>) {
    self.points_canvas.push(point);
  }

  pub fn points(&self) -> &[na::Point2<f32>] {
    &self.points_canvas
  }

  pub fn width_multiplier(&self) -> f32 {
    self.width_multiplier
  }

  pub fn color(&self) -> LinSrgba {
    self.color
  }
}
