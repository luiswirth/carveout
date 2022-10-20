use crate::{gfx::BufferSized, stroke::StrokeManager};

use std::mem;

pub struct StrokeRenderer {
  pipeline: wgpu::RenderPipeline,
  bind_group: wgpu::BindGroup,
}

impl StrokeRenderer {
  pub fn init(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    camera_buffer: &BufferSized,
  ) -> Self {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("stroke_renderer_bind_group_layout"),
      entries: &[wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::VERTEX,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: Some(camera_buffer.size),
        },
        count: None,
      }],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("stroke_renderer_bind_group"),
      layout: &bind_group_layout,
      entries: &[wgpu::BindGroupEntry {
        binding: 0,
        resource: camera_buffer.buffer.as_entire_binding(),
      }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("stroke_renderer_pipeline_layout"),
      bind_group_layouts: &[&bind_group_layout],
      push_constant_ranges: &[],
    });

    let fragment_targets = &[Some(wgpu::ColorTargetState {
      format,
      blend: Some(wgpu::BlendState::ALPHA_BLENDING),
      write_mask: wgpu::ColorWrites::ALL,
    })];

    let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

    let pipeline_descriptor = wgpu::RenderPipelineDescriptor {
      label: Some("stroke_render_pipeline"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[StrokeVertex::vertex_buffer_layout()],
      },
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: None,
        unclipped_depth: false,
        polygon_mode: wgpu::PolygonMode::Fill,
        conservative: false,
      },
      depth_stencil: None,
      multisample: wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: fragment_targets,
      }),

      multiview: None,
    };

    let pipeline = device.create_render_pipeline(&pipeline_descriptor);

    Self {
      pipeline,
      bind_group,
    }
  }

  pub fn render<'rp>(
    &'rp self,
    render_pass: &mut wgpu::RenderPass<'rp>,
    stroke_manager: &'rp StrokeManager,
  ) {
    render_pass.set_pipeline(&self.pipeline);
    render_pass.set_bind_group(0, &self.bind_group, &[]);

    for mesh in stroke_manager.data().meshes.values() {
      mesh.draw(render_pass);
    }
  }
}

pub type StrokeMeshCpu = crate::gfx::mesh::MeshCpu<StrokeVertex>;
pub type StrokeMeshGpu = crate::gfx::mesh::MeshGpu;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StrokeVertex {
  pub position: [f32; 2],
  pub normal: [f32; 2],
  pub stroke_width: f32,
  pub color: [f32; 4],
}

impl StrokeVertex {
  const LAYOUT_ATTRIBUTES: [wgpu::VertexAttribute; 4] =
    wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32, 3 => Float32x4];

  fn vertex_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
      array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &Self::LAYOUT_ATTRIBUTES,
    }
  }
}
