use crate::{canvas::gfx::CameraWithScreen, gfx::tessellate::TessellationStore};

use encase::UniformBuffer;
use palette::LinSrgba;
use std::mem;

/// Renders the drawed lines
pub struct StrokeRenderer {
  pipeline: wgpu::RenderPipeline,
  bind_group: wgpu::BindGroup,
  camera_ubo: wgpu::Buffer,
}

impl StrokeRenderer {
  pub fn init(device: &wgpu::Device) -> Self {
    let camera_ubo_size = 48;
    let camera_ubo = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("stroke_renderer_camera_ubo"),
      size: camera_ubo_size,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("stroke_renderer_bind_group_layout"),
      entries: &[wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::VERTEX,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: wgpu::BufferSize::new(camera_ubo_size),
        },
        count: None,
      }],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("stroke_renderer_bing_group"),
      layout: &bind_group_layout,
      entries: &[wgpu::BindGroupEntry {
        binding: 0,
        resource: camera_ubo.as_entire_binding(),
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
      camera_ubo,
    }
  }

  pub fn render<'a>(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    camera_screen: &CameraWithScreen,
    tessellated_strokes: impl IntoIterator<Item = &'a mut TessellationStore<Vertex>>,
  ) {
    let view: na::Transform2<f32> = na::convert(camera_screen.view_transform());
    let projection: na::Transform2<f32> = na::convert(camera_screen.projection());
    let view_projection = projection * view;
    let view_projection = view_projection.to_homogeneous();
    let camera_uniform = CameraUniform { view_projection };

    let mut buffer = UniformBuffer::new(Vec::new());
    buffer.write(&camera_uniform).unwrap();
    let byte_buffer = buffer.into_inner();
    queue.write_buffer(&self.camera_ubo, 0, &byte_buffer);

    let view = camera_screen.render_target();
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("stroke render pass"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: &view,
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
      stroke.render(device, queue, &mut pass);
    }
  }
}

#[derive(encase::ShaderType)]
struct CameraUniform {
  view_projection: na::Matrix3<f32>,
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
  const LAYOUT_ATTRIBUTES: [wgpu::VertexAttribute; 4] =
    wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32, 3 => Float32x4];

  fn vertex_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
      array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &Self::LAYOUT_ATTRIBUTES,
    }
  }
}

#[derive(Copy, Clone)]
pub struct VertexConstructor {
  pub width_multiplier: f32,
  pub color: LinSrgba,
}
