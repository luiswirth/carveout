mod backend;
mod render_target;

mod sidebar;

pub use self::{
  backend::{Backend, Platform, Renderer},
  render_target::UiRenderTarget,
};

use self::sidebar::SidebarUi;

use crate::canvas::Canvas;

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

  pub fn handle_event(&mut self, event: &crate::Event) {
    // ignored at the moment, because egui is too aggressive about having exclusive
    // access to an event.
    let _ = self.backend.handle_event(event);
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
        self.sidebar.ui(ctx, canvas);
        canvas.camera_mut().ui(ctx, device, renderer);
      },
    );
  }

  pub fn renderer_mut(&mut self) -> &mut Renderer {
    self.backend.renderer_mut()
  }
}
