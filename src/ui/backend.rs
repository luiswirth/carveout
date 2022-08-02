pub use egui_wgpu as renderer;
pub use egui_winit as platform;

pub use platform::State as Platform;
pub use renderer::renderer::RenderPass as Renderer;

use egui::Context as Ctx;
use winit::window;

pub struct Backend {
  ctx: Ctx,
  platform: Platform,
  renderer: Renderer,
}

impl Backend {
  pub fn new(event_loop: &crate::EventLoop, device: &wgpu::Device) -> Self {
    let platform = Platform::new(event_loop);
    let renderer = Renderer::new(device, crate::gfx::STANDARD_TEXTURE_FORMAT, 1);
    let ctx = Ctx::default();

    Self {
      ctx,
      platform,
      renderer,
    }
  }

  /// output bool indicates if egui wants exclusive access to this event
  pub fn handle_event<T>(&mut self, event: &winit::event::Event<T>) -> bool {
    match event {
      winit::event::Event::WindowEvent { event, .. } => self.platform.on_event(&self.ctx, event),
      _ => false,
    }
  }

  #[allow(clippy::too_many_arguments)]
  pub fn render<F>(
    &mut self,
    window: &window::Window,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    render_target: &wgpu::TextureView,
    load_operation: wgpu::LoadOp<wgpu::Color>,
    build_ui: F,
  ) where
    F: FnOnce(&Ctx, &mut Renderer),
  {
    let screen_descriptor = {
      let size = window.inner_size();
      renderer::renderer::ScreenDescriptor {
        size_in_pixels: [size.width, size.height],
        pixels_per_point: window.scale_factor() as f32,
      }
    };

    let raw_input: egui::RawInput = self.platform.take_egui_input(window);
    let full_output = self.ctx.run(raw_input, |ctx| {
      build_ui(ctx, &mut self.renderer);
    });
    self
      .platform
      .handle_platform_output(window, &self.ctx, full_output.platform_output);

    let clipped_primitives = self.ctx().tessellate(full_output.shapes);

    self
      .renderer
      .update_buffers(device, queue, &clipped_primitives, &screen_descriptor);
    for (tex_id, img_delta) in full_output.textures_delta.set {
      self
        .renderer
        .update_texture(device, queue, tex_id, &img_delta);
    }
    for tex_id in full_output.textures_delta.free {
      self.renderer.free_texture(&tex_id);
    }

    let clear_color = match load_operation {
      wgpu::LoadOp::Clear(c) => Some(c),
      wgpu::LoadOp::Load => None,
    };

    self.renderer.execute(
      encoder,
      render_target,
      &clipped_primitives,
      &screen_descriptor,
      clear_color,
    );
  }

  pub fn ctx(&self) -> &Ctx {
    &self.ctx
  }

  #[allow(dead_code)]
  pub fn platform(&self) -> &Platform {
    &self.platform
  }

  #[allow(dead_code)]
  pub fn platform_mut(&mut self) -> &mut Platform {
    &mut self.platform
  }

  #[allow(dead_code)]
  pub fn renderer(&self) -> &Renderer {
    &self.renderer
  }

  pub fn renderer_mut(&mut self) -> &mut Renderer {
    &mut self.renderer
  }
}
