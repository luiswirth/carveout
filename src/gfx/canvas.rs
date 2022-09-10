use super::{pdf::PdfRenderer, stroke::StrokeRenderer, BufferSized};

use crate::{
  pdf::PdfManager,
  spaces::{Space, SpaceManager},
  stroke::StrokeManager,
};

use encase::{ShaderType, UniformBuffer};

pub struct CanvasRenderer {
  pdf_renderer: PdfRenderer,
  stroke_renderer: StrokeRenderer,
  camera_buffer: BufferSized,
}

impl CanvasRenderer {
  pub fn init(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
    let camera_buffer_size = CameraUniform::min_size();
    let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("stroke_renderer_camera_ubo"),
      size: camera_buffer_size.into(),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let camera_buffer = BufferSized::new(camera_buffer, camera_buffer_size);

    let pdf_renderer = PdfRenderer::init(device, format);
    let stroke_renderer = StrokeRenderer::init(device, format, &camera_buffer);

    Self {
      pdf_renderer,
      stroke_renderer,
      camera_buffer,
    }
  }

  pub fn prepare(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    spaces: &SpaceManager,
    pdf_manager: Option<&PdfManager>,
  ) {
    let view: na::Transform2<f32> = na::convert(spaces.canvas_to_view());
    let projection: na::Transform2<f32> = na::convert(spaces.canvas_view_to_screen_norm());
    let view_projection = projection * view;
    let view_projection = view_projection.to_homogeneous();
    let camera_uniform = CameraUniform { view_projection };

    let mut buffer = UniformBuffer::new(Vec::new());
    buffer.write(&camera_uniform).unwrap();
    let byte_buffer = buffer.into_inner();
    queue.write_buffer(&self.camera_buffer.buffer, 0, &byte_buffer);

    if let Some(pdf_manager) = pdf_manager {
      self
        .pdf_renderer
        .prepare(device, queue, spaces, pdf_manager);
    }
  }

  pub fn render<'rp>(
    &'rp self,
    render_pass: &mut wgpu::RenderPass<'rp>,
    spaces: &SpaceManager,
    stroke_manager: &'rp StrokeManager,
  ) {
    let viewport = spaces.transform_rect(
      spaces.screen_rect_window_logical(),
      Space::WindowLogical,
      Space::WindowPhysical,
    );
    render_pass.set_viewport(
      viewport.min().x,
      viewport.min().y,
      viewport.size().x,
      viewport.size().y,
      0.0,
      1.0,
    );

    self.pdf_renderer.render(render_pass);
    self.stroke_renderer.render(render_pass, stroke_manager);
  }
}

#[derive(ShaderType)]
struct CameraUniform {
  view_projection: na::Matrix3<f32>,
}
