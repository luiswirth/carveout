mod eraser;
mod pen;
mod rotate_tool;
mod select_loop;
mod translate_tool;
mod zoom_tool;

pub use self::select_loop::SelectLoop;
use self::{
  eraser::update_eraser, pen::Pen, rotate_tool::update_rotate_tool,
  translate_tool::update_translate_tool, zoom_tool::update_zoom_tool,
};

use crate::{camera::Camera, content::ContentManager, input::InputManager, stroke::StrokeManager};

use palette::LinSrgb;

#[derive(Default)]
pub struct ToolManager {
  pub selected: ToolEnum,
  pub configs: ToolConfigs,

  pub pen: Pen,
  pub select_loop: SelectLoop,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum ToolEnum {
  #[default]
  Pen,
  Eraser,

  SelectLoop,

  Translate,
  Rotate,
  Zoom,
}

#[derive(Default)]
pub struct ToolConfigs {
  pub pen: PenConfig,
}

#[derive(Clone)]
pub struct PenConfig {
  pub width: f32,
  pub color: LinSrgb,
}
impl Default for PenConfig {
  fn default() -> Self {
    Self {
      width: 1.0,
      color: palette::named::WHITE.into_format().into_linear(),
    }
  }
}

impl ToolManager {
  pub fn update(
    &mut self,
    camera: &mut Camera,
    input: &InputManager,
    content_manager: &mut ContentManager,
    stroke_manager: &StrokeManager,
  ) {
    match self.selected {
      ToolEnum::Pen => self
        .pen
        .update(input, content_manager, &self.configs.pen, camera),
      ToolEnum::Eraser => update_eraser(input, content_manager, stroke_manager),
      ToolEnum::SelectLoop => {
        self
          .select_loop
          .update(camera, input, content_manager, stroke_manager)
      }
      ToolEnum::Translate => update_translate_tool(input, camera),
      ToolEnum::Rotate => update_rotate_tool(input, camera),
      ToolEnum::Zoom => update_zoom_tool(input, camera),
    }
  }
}
