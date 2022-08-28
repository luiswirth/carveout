pub mod content;
pub mod tool;

mod gfx;
mod input;
mod space;
mod stroke;

use self::{
  content::ContentManager, gfx::CameraWithScreen, input::InputHandler, stroke::StrokeManager,
  tool::ToolConfig,
};

use crate::ui::CanvasScreen;

use std::{cell::RefCell, rc::Rc};

pub struct CanvasManager {
  content: ContentManager,
  camera_screen: CameraWithScreen,
  tool_config: ToolConfig,
  input_handler: InputHandler,

  stroke_manager: StrokeManager,
}

impl CanvasManager {
  pub fn init(device: &wgpu::Device, screen: Rc<RefCell<CanvasScreen>>) -> Self {
    let content = ContentManager::default();
    let camera_screen = CameraWithScreen::init(screen);
    let input_handler = InputHandler::default();
    let tool_config = ToolConfig::default();

    let stroke_manager = StrokeManager::init(device);

    Self {
      content,
      camera_screen,
      input_handler,
      tool_config,

      stroke_manager,
    }
  }

  pub fn handle_event(&mut self, event: &crate::Event, window: &winit::window::Window) {
    self
      .input_handler
      .handle_event(event, window, &mut self.camera_screen);
  }

  pub fn update(&mut self) {
    self.input_handler.update(
      &self.tool_config,
      &mut self.content,
      &self.stroke_manager,
      &mut self.camera_screen,
    );

    let access = self.content.access();
    let delta = self.content.delta();
    self.stroke_manager.update_strokes(access, &delta.strokes);
    self.content.clear_delta();
  }

  pub fn render(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
  ) {
    self
      .stroke_manager
      .render(device, queue, encoder, &self.camera_screen);
  }

  pub fn content(&self) -> &ContentManager {
    &self.content
  }

  pub fn content_mut(&mut self) -> &mut ContentManager {
    &mut self.content
  }

  pub fn camera_screen_mut(&mut self) -> &mut CameraWithScreen {
    &mut self.camera_screen
  }

  pub fn tool_config_mut(&mut self) -> &mut ToolConfig {
    &mut self.tool_config
  }
}
