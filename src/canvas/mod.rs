pub mod content;
pub mod protocol;
pub mod tool;

mod gfx;
mod input;
mod space;
mod stroke;

use self::{
  content::CanvasContent, gfx::CameraWithScreen, input::InputHandler, protocol::ProtocolManager,
  stroke::StrokeManager, tool::ToolConfig,
};

use crate::ui::CanvasScreen;

use std::{cell::RefCell, rc::Rc};

pub struct CanvasManager {
  content: CanvasContent,
  protocol_manager: ProtocolManager,
  camera_screen: CameraWithScreen,
  tool_config: ToolConfig,
  input_handler: InputHandler,

  stroke_manager: StrokeManager,
}

impl CanvasManager {
  pub fn init(device: &wgpu::Device, screen: Rc<RefCell<CanvasScreen>>) -> Self {
    let content = CanvasContent::default();
    let protocol_manager = ProtocolManager::default();
    let camera_screen = CameraWithScreen::init(screen);
    let input_handler = InputHandler::default();
    let tool_config = ToolConfig::default();

    let stroke_manager = StrokeManager::init(device);

    Self {
      content,
      protocol_manager,
      camera_screen,
      input_handler,
      tool_config,

      stroke_manager,
    }
  }

  pub fn handle_event(&mut self, event: &crate::Event, window: &winit::window::Window) {
    self.input_handler.handle_event(
      event,
      window,
      &mut self.camera_screen,
      &self.tool_config,
      &mut self.protocol_manager,
      &mut self.content,
      &self.stroke_manager,
    );
  }

  pub fn update(&mut self) {
    self.protocol_manager.update(self.content.persistent_mut());

    // TODO: don't set all strokes every frame, only update when changed
    self.stroke_manager.clear_strokes();
    let (ongoing, persistent) = self.content.ongoing_persistent_mut();
    let strokes = persistent.strokes().iter().chain(&ongoing.stroke);
    self.stroke_manager.update_strokes(strokes);
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

  pub fn content(&self) -> &CanvasContent {
    &self.content
  }

  pub fn content_mut(&mut self) -> &mut CanvasContent {
    &mut self.content
  }

  pub fn protocol_manager(&self) -> &ProtocolManager {
    &self.protocol_manager
  }

  pub fn protocol_manager_mut(&mut self) -> &mut ProtocolManager {
    &mut self.protocol_manager
  }

  pub fn camera_screen_mut(&mut self) -> &mut CameraWithScreen {
    &mut self.camera_screen
  }

  pub fn tool_config_mut(&mut self) -> &mut ToolConfig {
    &mut self.tool_config
  }
}
