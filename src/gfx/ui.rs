use egui::ClippedPrimitive;
use egui_wgpu::renderer::{RenderPass as EguiRenderPass, ScreenDescriptor};
use winit::window::Window;

pub struct UiRenderer {
  egui_renderer: EguiRenderPass,
  screen_descriptor: ScreenDescriptor,
  primitives: Vec<ClippedPrimitive>,
}

impl UiRenderer {
  pub fn init(device: &wgpu::Device) -> Self {
    let egui_renderer =
      EguiRenderPass::new(device, super::STANDARD_TEXTURE_FORMAT, super::MSAA_NSAMPLES);
    let screen_descriptor = ScreenDescriptor {
      size_in_pixels: [0; 2],
      pixels_per_point: 0.0,
    };
    let primitives = Vec::default();

    Self {
      egui_renderer,
      screen_descriptor,
      primitives,
    }
  }

  pub fn prepare(
    &mut self,
    window: &Window,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    egui_ctx: &egui::Context,
    shapes: Vec<egui::epaint::ClippedShape>,
    textures_delta: egui::TexturesDelta,
  ) {
    self.screen_descriptor = {
      let size = window.inner_size();
      let scale_factor = window.scale_factor() as f32;
      ScreenDescriptor {
        size_in_pixels: [size.width, size.height],
        pixels_per_point: scale_factor,
      }
    };

    self.primitives = egui_ctx.tessellate(shapes);

    self
      .egui_renderer
      .update_buffers(device, queue, &self.primitives, &self.screen_descriptor);
    for (tex_id, img_delta) in textures_delta.set {
      self
        .egui_renderer
        .update_texture(device, queue, tex_id, &img_delta);
    }
    for tex_id in textures_delta.free {
      self.egui_renderer.free_texture(&tex_id);
    }
  }

  pub fn render<'rp>(&'rp mut self, render_pass: &mut wgpu::RenderPass<'rp>) {
    self.egui_renderer.execute_with_renderpass(
      render_pass,
      &self.primitives,
      &self.screen_descriptor,
    )
  }
}
