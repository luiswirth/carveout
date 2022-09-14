use super::stroke::StrokeRenderer;

use crate::{spaces::SpaceManager, stroke::StrokeManager};

pub struct CanvasRenderer {
  stroke_renderer: StrokeRenderer,
}

impl CanvasRenderer {
  pub fn init(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
    let stroke_renderer = StrokeRenderer::init(device, format);
    Self { stroke_renderer }
  }

  pub fn prepare(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, spaces: &SpaceManager) {
    self.stroke_renderer.prepare(queue, spaces);
  }

  pub fn render<'rp>(
    &'rp self,
    render_pass: &mut wgpu::RenderPass<'rp>,
    spaces: &SpaceManager,
    stroke_manager: &'rp StrokeManager,
  ) {
    let viewport = spaces.screen_rect().window_physical();
    render_pass.set_viewport(
      viewport.min().x,
      viewport.min().y,
      viewport.size().x,
      viewport.size().y,
      0.0,
      1.0,
    );

    self.stroke_renderer.render(render_pass, stroke_manager);
  }
}
