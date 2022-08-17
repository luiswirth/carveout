mod backend;
mod render_target;

mod canvas;
mod sidebar;

pub use self::canvas::CanvasScreen;

use self::{backend::Backend, canvas::CanvasUi, sidebar::SidebarUi};

pub struct Ui {
  backend: Backend,

  sidebar: SidebarUi,
  canvas: CanvasUi,
}

impl Ui {
  pub fn init(event_loop: &crate::EventLoop, device: &wgpu::Device) -> Self {
    let mut backend = Backend::new(event_loop, device);

    let sidebar = SidebarUi::init();
    let canvas = CanvasUi::init(device, backend.renderer_mut());

    Self {
      backend,

      sidebar,
      canvas,
    }
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

    canvas_manager: &mut crate::CanvasManager,
  ) {
    self.backend.render(
      window,
      device,
      queue,
      encoder,
      surface_view,
      wgpu::LoadOp::Clear(wgpu::Color::BLACK),
      |ctx, renderer| {
        self.sidebar.ui(ctx, canvas_manager);
        self.canvas.ui(ctx, device, renderer);
      },
    );
  }

  pub fn canvas(&self) -> &CanvasUi {
    &self.canvas
  }
}
