pub mod tool;

mod camera;
mod input_handler;
mod stroke;

pub use self::camera::Camera;

use self::{input_handler::InputHandler, stroke::StrokeManager, tool::ToolConfig};

pub struct Canvas {
  camera: Camera,
  input_handler: InputHandler,
  tool_config: ToolConfig,

  stroke_manager: StrokeManager,
}

impl Canvas {
  pub fn init(device: &wgpu::Device, ui_renderer: &mut crate::ui::Renderer) -> Self {
    let camera = Camera::init(device, ui_renderer);
    let input_handler = InputHandler::default();
    let tool_config = ToolConfig::default();

    let stroke_manager = StrokeManager::init(device);

    Self {
      camera,
      input_handler,
      tool_config,

      stroke_manager,
    }
  }

  pub fn handle_event(&mut self, event: &crate::Event, window: &winit::window::Window) {
    self.input_handler.handle_event(
      event,
      window,
      &mut self.camera,
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
      .render(device, queue, encoder, &self.camera);
  }

  pub fn camera_mut(&mut self) -> &mut Camera {
    &mut self.camera
  }

  pub fn tool_config_mut(&mut self) -> &mut ToolConfig {
    &mut self.tool_config
  }
}
