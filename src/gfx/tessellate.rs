use crate::gfx::util::DynamicBuffer;
use std::{
  convert::{TryFrom, TryInto},
  mem,
};

use lyon::tessellation::{
  FillGeometryBuilder, FillVertexConstructor, GeometryBuilder, StrokeGeometryBuilder,
  StrokeVertexConstructor, VertexId,
};

type Index = u32;

pub struct TessellationStore<Vertex> {
  vertices: Vec<Vertex>,
  indices: Vec<Index>,

  vertex_buffer: Option<DynamicBuffer>,
  index_buffer: Option<DynamicBuffer>,
}

impl<V> Default for TessellationStore<V> {
  fn default() -> Self {
    let vertices = Vec::new();
    let indices = Vec::new();
    let vertex_buffer = None;
    let index_buffer = None;
    Self {
      vertices,
      indices,
      vertex_buffer,
      index_buffer,
    }
  }
}

impl<Vertex> TessellationStore<Vertex>
where
  Vertex: bytemuck::Pod,
{
  fn update_buffers(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
    let vertex_buffer = self.vertex_buffer.get_or_insert(DynamicBuffer::new(
      device,
      &wgpu::BufferDescriptor {
        label: Some("tesselation_vertex_buffer"),
        size: wgpu::COPY_BUFFER_ALIGNMENT,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      },
    ));
    let index_buffer = self.index_buffer.get_or_insert(DynamicBuffer::new(
      device,
      &wgpu::BufferDescriptor {
        label: Some("tesselation_index_buffer"),
        size: wgpu::COPY_BUFFER_ALIGNMENT,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      },
    ));

    vertex_buffer.upload(device, queue, bytemuck::cast_slice(&self.vertices));
    index_buffer.upload(device, queue, bytemuck::cast_slice(&self.indices));
  }

  pub fn render<'s, 'ce>(
    &'s mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    render_pass: &mut wgpu::RenderPass<'ce>,
  ) where
    's: 'ce,
  {
    if self.vertices.is_empty() || self.indices.is_empty() {
      return;
    }
    self.update_buffers(device, queue);

    if let Some((vertex_buffer, index_buffer)) =
      self.vertex_buffer.as_ref().zip(self.index_buffer.as_ref())
    {
      render_pass.set_vertex_buffer(
        0,
        vertex_buffer
          .raw()
          .slice(0..(self.vertices.len() * mem::size_of::<Vertex>()) as u64),
      );
      render_pass.set_index_buffer(
        index_buffer
          .raw()
          .slice(0..(self.indices.len() * mem::size_of::<Index>()) as u64),
        wgpu::IndexFormat::Uint32,
      );

      render_pass.draw_indexed(
        0..(u32::try_from(self.indices.len()).expect("too many indicies")),
        0,
        0..1,
      );
    }
  }
}

pub struct TessellationStoreBuilder<'s, Vertex, Constructor> {
  store: &'s mut TessellationStore<Vertex>,
  voffset: usize,
  ioffset: usize,
  constructor: Constructor,
}

impl<'s, V, C> GeometryBuilder for TessellationStoreBuilder<'s, V, C> {
  fn begin_geometry(&mut self) {
    self.voffset = self.store.vertices.len();
    self.ioffset = self.store.indices.len();
  }

  fn add_triangle(&mut self, a: VertexId, b: VertexId, c: VertexId) {
    self
      .store
      .indices
      .push(a.0 + u32::try_from(self.voffset).unwrap());
    self
      .store
      .indices
      .push(b.0 + u32::try_from(self.voffset).unwrap());
    self
      .store
      .indices
      .push(c.0 + u32::try_from(self.voffset).unwrap());
  }

  fn abort_geometry(&mut self) {
    self.store.vertices.truncate(self.voffset);
    self.store.indices.truncate(self.ioffset);
  }
}

impl<'s, Vertex, VertexConstructor> FillGeometryBuilder
  for TessellationStoreBuilder<'s, Vertex, VertexConstructor>
where
  VertexConstructor: FillVertexConstructor<Vertex>,
{
  fn add_fill_vertex(
    &mut self,
    lyon_vertex: lyon::lyon_tessellation::FillVertex<'_>,
  ) -> Result<VertexId, lyon::lyon_tessellation::GeometryBuilderError> {
    self
      .store
      .vertices
      .push(self.constructor.new_vertex(lyon_vertex));
    let len = self.store.vertices.len();
    if len > Index::MAX as usize {
      return Err(lyon::lyon_tessellation::GeometryBuilderError::TooManyVertices);
    }
    Ok(VertexId((len - 1 - self.voffset).try_into().unwrap()))
  }
}

impl<'s, Vertex, VertexConstructor> StrokeGeometryBuilder
  for TessellationStoreBuilder<'s, Vertex, VertexConstructor>
where
  VertexConstructor: StrokeVertexConstructor<Vertex>,
{
  fn add_stroke_vertex(
    &mut self,
    lyon_vertex: lyon::lyon_tessellation::StrokeVertex<'_, '_>,
  ) -> Result<VertexId, lyon::lyon_tessellation::GeometryBuilderError> {
    self
      .store
      .vertices
      .push(self.constructor.new_vertex(lyon_vertex));
    let len = self.store.vertices.len();
    if len > Index::MAX as usize {
      return Err(lyon::lyon_tessellation::GeometryBuilderError::TooManyVertices);
    }
    Ok(VertexId((len - 1 - self.voffset).try_into().unwrap()))
  }
}

impl<'s, V, C> TessellationStoreBuilder<'s, V, C> {
  pub fn new(store: &'s mut TessellationStore<V>, constructor: C) -> Self {
    let voffset = store.vertices.len();
    let ioffset = store.indices.len();
    Self {
      store,
      voffset,
      ioffset,
      constructor,
    }
  }
}
