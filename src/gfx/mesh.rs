use wgpu::util::DeviceExt;

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
