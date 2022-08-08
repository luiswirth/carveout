pub mod tool;

mod input_handler;
mod portal;
mod stroke;

pub use self::portal::CanvasPortal;

use self::{input_handler::InputHandler, stroke::StrokeManager, tool::ToolConfig};

pub struct Canvas {
  portal: CanvasPortal,
  input_handler: InputHandler,
  tool_config: ToolConfig,

  stroke_manager: StrokeManager,
}

impl Canvas {
  pub fn init(device: &wgpu::Device, ui_renderer: &mut crate::ui::Renderer) -> Self {
    let portal = CanvasPortal::init(device, ui_renderer);
    let input_handler = InputHandler::default();
    let tool_config = ToolConfig::default();

    let stroke_manager = StrokeManager::init(device);

    Self {
      portal,
      input_handler,
      tool_config,

      stroke_manager,
    }
  }

  pub fn handle_event(&mut self, event: &crate::Event, window: &winit::window::Window) {
    self.input_handler.handle_event(
      event,
      window,
      &mut self.portal,
      &self.tool_config,
      &mut self.stroke_manager,
    );
  }

  pub fn render(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
  ) {
    self
      .stroke_manager
      .render(device, queue, encoder, &self.portal);
  }

  pub fn portal_mut(&mut self) -> &mut CanvasPortal {
    &mut self.portal
  }

  pub fn tool_config_mut(&mut self) -> &mut ToolConfig {
    &mut self.tool_config
  }
}
