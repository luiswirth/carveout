use super::space::{CanvasPoint, CanvasPointExt, ScreenPixelPoint};

use crate::ui::CanvasScreen;

use std::{
  cell::{Ref, RefCell},
  rc::Rc,
};
use wgpu::util::DeviceExt;

pub struct CameraWithScreen {
  camera: Camera,
  screen: Rc<RefCell<CanvasScreen>>,
}

impl CameraWithScreen {
  pub fn init(screen: Rc<RefCell<CanvasScreen>>) -> Self {
    let camera = Camera::init();
    Self { camera, screen }
  }

  pub fn camera_mut(&mut self) -> &mut Camera {
    &mut self.camera
  }

  pub fn render_target(&self) -> Ref<wgpu::TextureView> {
    Ref::map(self.screen.borrow(), |s| s.render_target())
  }

  pub fn screen_rect(&self) -> egui::Rect {
    self.screen.borrow().rect()
  }
}

impl CameraWithScreen {
  // view transform (mVp)
  pub fn canvas_to_view(&self) -> na::IsometryMatrix2<f32> {
    let translation = na::Translation2::from(-self.camera.position.cast());
    let rotation = na::Rotation2::new(-self.camera.angle);
    rotation * translation
  }

  // projection (mvP)
  pub fn view_to_screen_norm(&self) -> na::Scale2<f32> {
    let camera_scale = self.camera.scale;
    let camera_scale = na::Scale2::new(camera_scale, camera_scale);
    let screen_scale = self.screen_rect().size();
    let screen_scale = na::Scale2::new(2.0 / screen_scale.x, 2.0 / screen_scale.y);
    screen_scale * camera_scale
  }

  // viewport transform
  pub fn screen_norm_to_pixel(&self) -> na::Affine2<f32> {
    let translation = na::Translation2::new(1.0, 1.0);
    let translation: na::Affine2<f32> = na::convert(translation);
    let scale = self.screen_rect().size();
    let scale = na::Scale2::new(scale.x / 2.0, scale.y / 2.0);
    let scale: na::Affine2<f32> = na::convert(scale);
    scale * translation
  }
}
impl CameraWithScreen {
  pub fn rotate_with_center(&mut self, angle: f32, center: ScreenPixelPoint) {
    let center = CanvasPoint::from_screen_pixel(center, self);
    let mut vector = self.camera.position - center;
    let rotation = na::Rotation2::new(angle);
    vector = rotation.transform_vector(&vector.cast()).cast();

    self.camera.position = center + vector;
    self.camera.angle = (self.camera.angle + angle).rem_euclid(std::f32::consts::TAU);
  }

  pub fn scale_with_center(&mut self, scale: f32, center: ScreenPixelPoint) {
    let center = CanvasPoint::from_screen_pixel(center, self);
    let mut vector = self.camera.position - center;
    vector.scale_mut((1.0 / scale).into());

    self.camera.position = center + vector;
    self.camera.scale *= scale;
  }
}

pub struct Camera {
  pub position: CanvasPoint,
  pub angle: f32,
  pub scale: f32,
}

impl Camera {
  pub fn init() -> Self {
    let position = CanvasPoint::origin();
    let angle = 0.0;
    let scale = 1.0;

    Self {
      position,
      angle,
      scale,
    }
  }
}

pub type Tessellation<Vertex> = lyon::tessellation::VertexBuffers<Vertex, u32>;

pub struct MeshCpu<Vertex> {
  vertices: Vec<Vertex>,
  indices: Vec<u32>,
}
impl<Vertex> MeshCpu<Vertex> {
  pub fn from_tessellation(tessellation: Tessellation<Vertex>) -> Self {
    let vertices = tessellation.vertices;
    let indices = tessellation.indices;
    Self { vertices, indices }
  }

  pub fn vertices(&self) -> &[Vertex] {
    &self.vertices
  }
  pub fn indices(&self) -> &[u32] {
    &self.indices
  }
}

pub struct MeshGpu {
  vertex_buffer: wgpu::Buffer,
  index_buffer: wgpu::Buffer,
  index_count: u32,
}
impl MeshGpu {
  pub fn from_mesh_cpu<Vertex: bytemuck::Pod>(
    mesh_cpu: &MeshCpu<Vertex>,
    device: &wgpu::Device,
  ) -> Self {
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("stroke_mesh_vertices"),
      contents: bytemuck::cast_slice(&mesh_cpu.vertices),
      usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("stroke_mesh_indicies"),
      contents: bytemuck::cast_slice(&mesh_cpu.indices),
      usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
    });
    let index_count = u32::try_from(mesh_cpu.indices.len()).unwrap();

    Self {
      vertex_buffer,
      index_buffer,
      index_count,
    }
  }

  pub fn draw<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
    let Self {
      ref vertex_buffer,
      ref index_buffer,
      index_count,
    } = *self;
    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    render_pass.draw_indexed(0..index_count, 0, 0..1);
  }
}
