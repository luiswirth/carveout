pub mod tool;

mod input_handler;
mod stroke;
mod ui;

pub use ui::CanvasUi;

use self::{input_handler::InputHandler, stroke::StrokeManager, tool::ToolConfig};

pub struct Canvas {
  ui: CanvasUi,
  viewport: CanvasViewport,
  input_handler: InputHandler,
  tool_config: ToolConfig,

  stroke_manager: StrokeManager,
}

impl Canvas {
  pub fn init(device: &wgpu::Device, ui_renderer: &mut crate::ui::Renderer) -> Self {
    let ui = CanvasUi::init(device, ui_renderer);
    let viewport = CanvasViewport::default();
    let input_handler = InputHandler::default();
    let tool_config = ToolConfig::default();

    let stroke_manager = StrokeManager::init(device);

    Self {
      ui,
      viewport,
      input_handler,
      tool_config,

      stroke_manager,
    }
  }

  pub fn handle_event(&mut self, event: &crate::Event, window: &winit::window::Window) {
    self.input_handler.handle_event(
      event,
      window,
      &mut self.viewport,
      self.ui.ui_box(),
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
    self.stroke_manager.render(
      device,
      queue,
      encoder,
      self.ui.render_target(),
      &self.viewport,
    );
  }

  pub fn ui_mut(&mut self) -> &mut CanvasUi {
    &mut self.ui
  }

  pub fn tool_config_mut(&mut self) -> &mut ToolConfig {
    &mut self.tool_config
  }
}

#[derive(Default)]
pub struct CanvasViewport {
  pub transform: euclid::Transform2D<
    f32,
    crate::util::space::CanvasSpace,
    crate::util::space::CanvasViewportSpace,
  >,
}
