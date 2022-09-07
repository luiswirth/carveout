mod canvas;
mod overlay;
mod sidebar;

use self::{canvas::CanvasUi, sidebar::SidebarUi};

#[derive(Default)]
pub struct Ui {
  sidebar: SidebarUi,
  canvas: CanvasUi,
}

impl Ui {
  pub fn run(&mut self, ctx: &egui::Context, mut ui_access: UiAccess) {
    self.sidebar.ui(ctx, &mut ui_access);
    self.canvas.ui(ctx, &mut ui_access);
  }

  pub fn canvas(&self) -> &CanvasUi {
    &self.canvas
  }
}

pub struct UiAccess<'a> {
  pub camera: &'a mut crate::Camera,
  pub content_manager: &'a mut crate::ContentManager,
  pub tool_manager: &'a mut crate::ToolManager,
  pub stroke_manager: &'a mut crate::StrokeManager,
}
