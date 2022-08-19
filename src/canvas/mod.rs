pub mod content;
pub mod tool;
pub mod undo;

mod gfx;
mod input;
mod space;
mod stroke;

use self::{
  content::{CanvasContent, PersistentContent},
  gfx::CameraWithScreen,
  input::InputHandler,
  stroke::StrokeManager,
  tool::ToolConfig,
  undo::UndoTree,
};

use crate::ui::CanvasScreen;

use std::{cell::RefCell, rc::Rc};

pub struct CanvasManager {
  content: CanvasContent,
  undo_tree: UndoTree,
  camera_screen: CameraWithScreen,
  tool_config: ToolConfig,
  input_handler: InputHandler,

  stroke_manager: StrokeManager,
}

impl CanvasManager {
  pub fn init(device: &wgpu::Device, screen: Rc<RefCell<CanvasScreen>>) -> Self {
    let content = CanvasContent::init();
    let undo_tree = UndoTree::new();
    let camera_screen = CameraWithScreen::init(screen);
    let input_handler = InputHandler::default();
    let tool_config = ToolConfig::default();

    let stroke_manager = StrokeManager::init(device);

    Self {
      content,
      undo_tree,
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
      &mut self.undo_tree,
      &mut self.content,
    );
  }

  pub fn render(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
  ) {
    let (ongoing, persistent) = self.content.ongoing_persistent_mut();

    self.stroke_manager.render(
      device,
      queue,
      encoder,
      &self.camera_screen,
      persistent.strokes_mut().iter_mut(),
      ongoing.stroke.as_mut(),
    );
  }

  pub fn camera_screen_mut(&mut self) -> &mut CameraWithScreen {
    &mut self.camera_screen
  }

  pub fn tool_config_mut(&mut self) -> &mut ToolConfig {
    &mut self.tool_config
  }

  pub fn undo_tree_content_mut(&mut self) -> (&mut UndoTree, &mut PersistentContent) {
    (&mut self.undo_tree, self.content.persistent_mut())
  }
}
