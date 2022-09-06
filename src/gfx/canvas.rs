use super::stroke::StrokeRenderer;

use crate::{camera::Camera, stroke::StrokeManager};

pub struct CanvasRenderer {
  stroke_renderer: StrokeRenderer,
}

impl CanvasRenderer {
  pub fn init(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
    let stroke_renderer = StrokeRenderer::init(device, format);
    Self { stroke_renderer }
  }

  pub fn prepare(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, camera: &Camera) {
    self.stroke_renderer.prepare(queue, camera);
  }

  pub fn render<'rp>(
    &'rp self,
    render_pass: &mut wgpu::RenderPass<'rp>,
    scale_factor: f32,
    camera: &Camera,
    stroke_manager: &'rp StrokeManager,
  ) {
    let mut viewport = camera.viewport;
    viewport.min = (viewport.min.to_vec2() * scale_factor).to_pos2();
    viewport.max = (viewport.max.to_vec2() * scale_factor).to_pos2();
    render_pass.set_viewport(
      viewport.min.x,
      viewport.min.y,
      viewport.size().x,
      viewport.size().y,
      0.0,
      1.0,
    );

    self.stroke_renderer.render(render_pass, stroke_manager);
  }
}
