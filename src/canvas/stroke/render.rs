use crate::util;

use super::{CanvasViewport, TessallatedStroke};

use palette::LinSrgba;
use std::{borrow::Cow, mem};

/// Renders the drawed lines
pub struct StrokeRenderer {
  pipeline: wgpu::RenderPipeline,
  bind_group: wgpu::BindGroup,
  viewport_ubo: wgpu::Buffer,
}

impl StrokeRenderer {
  pub fn init(device: &wgpu::Device) -> Self {
    let globals_ubo_size = mem::size_of::<Globals>() as wgpu::BufferAddress;
    let globals_ubo = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("stroke_renderer_globals_buffer"),
      size: globals_ubo_size,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("stroke/shader.wgsl"),
      source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("stroke_renderer_bind_group_layout"),
      entries: &[wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::VERTEX,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: wgpu::BufferSize::new(globals_ubo_size),
        },
        count: None,
      }],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("stroke_renderer_bing_group"),
      layout: &bind_group_layout,
      entries: &[wgpu::BindGroupEntry {
        binding: 0,
        resource: globals_ubo.as_entire_binding(),
      }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("stroke_renderer_pipeline_layout"),
      bind_group_layouts: &[&bind_group_layout],
      push_constant_ranges: &[],
    });

    let fragment_targets = &[Some(wgpu::ColorTargetState {
      format: crate::gfx::STANDARD_TEXTURE_FORMAT,
      blend: Some(wgpu::BlendState::REPLACE),
      write_mask: wgpu::ColorWrites::ALL,
    })];

    let pipeline_descriptor = wgpu::RenderPipelineDescriptor {
      label: Some("stroke_render_pipeline"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[Vertex::vertex_buffer_layout()],
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
      viewport_ubo: globals_ubo,
    }
  }

  pub fn render<'a>(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    render_target: &wgpu::TextureView,
    tessellated_strokes: impl IntoIterator<Item = &'a mut TessallatedStroke>,
    viewport: &CanvasViewport,
  ) {
    queue.write_buffer(
      &self.viewport_ubo,
      0,
      bytemuck::cast_slice(&[Globals {
        viewport_transform: viewport.transform,
      }]),
    );

    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: None,
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: render_target,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
          store: true,
        },
        resolve_target: None,
      })],
      depth_stencil_attachment: None,
    });

    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(0, &self.bind_group, &[]);

    for stroke in tessellated_strokes {
      stroke.0.render(device, queue, &mut pass);
    }
  }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
  viewport_transform:
    euclid::Transform2D<f32, util::space::CanvasSpace, util::space::CanvasViewportSpace>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
  pub position: [f32; 2],
  pub normal: [f32; 2],
  pub stroke_width: f32,
  pub color: [f32; 4],
}

impl Vertex {
  fn vertex_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
    // TODO: auto generate?
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<Vertex>() as u64,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &[
        // position
        wgpu::VertexAttribute {
          format: wgpu::VertexFormat::Float32x2,
          offset: 0,
          shader_location: 0,
        },
        // normal
        wgpu::VertexAttribute {
          format: wgpu::VertexFormat::Float32x2,
          offset: 8,
          shader_location: 1,
        },
        // stroke_width
        wgpu::VertexAttribute {
          format: wgpu::VertexFormat::Float32,
          offset: 16,
          shader_location: 2,
        },
        // color
        wgpu::VertexAttribute {
          format: wgpu::VertexFormat::Float32x4,
          offset: 20,
          shader_location: 3,
        },
      ],
    }
  }
}

#[derive(Copy, Clone)]
pub struct VertexConstructor {
  pub stroke_width: f32,
  pub color: LinSrgba,
}
