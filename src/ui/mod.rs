mod backend;
mod sidebar;

use crate::canvas::Canvas;

use self::sidebar::SidebarUi;

pub use self::backend::{Backend, Platform, Renderer};

pub struct Ui {
  backend: Backend,

  sidebar: SidebarUi,
}

impl Ui {
  pub fn init(event_loop: &crate::EventLoop, device: &wgpu::Device) -> Self {
    let backend = Backend::new(event_loop, device);
    let sidebar = SidebarUi::init();

    Self { backend, sidebar }
  }

  pub fn handle_event(&mut self, event: &crate::Event) -> bool {
    self.backend.handle_event(event)
  }

  pub fn render(
    &mut self,
    window: &crate::Window,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    surface_view: &wgpu::TextureView,

    canvas: &mut Canvas,
  ) {
    self.backend.render(
      window,
      device,
      queue,
      encoder,
      surface_view,
      wgpu::LoadOp::Clear(wgpu::Color::BLACK),
      |ctx, renderer| {
        self.sidebar.build_ui(ctx, canvas.tool_config_mut());
        canvas.ui_mut().build_ui(ctx, device, renderer);
      },
    );
  }

  pub fn renderer_mut(&mut self) -> &mut Renderer {
    self.backend.renderer_mut()
  }
}
