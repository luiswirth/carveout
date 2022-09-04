mod canvas;
mod overlay;
mod sidebar;

use self::{canvas::CanvasUi, sidebar::SidebarUi};

pub struct Ui {
  sidebar: SidebarUi,
  canvas: CanvasUi,
}

impl Ui {
  pub fn init() -> Self {
    let sidebar = SidebarUi::init();
    let canvas = CanvasUi::init();

    Self { sidebar, canvas }
  }

  pub fn run(&mut self, ctx: &egui::Context, mut ui_access: UiAccess) {
    self.sidebar.ui(ctx, &mut ui_access);
    self.canvas.ui(ctx, &mut ui_access);
  }

  pub fn canvas(&self) -> &CanvasUi {
    &self.canvas
  }
}

pub struct UiAccess<'a> {
  pub content_manager: &'a mut crate::ContentManager,
  pub camera: &'a mut crate::Camera,
  pub tool_config: &'a mut crate::ToolConfig,
  pub stroke_manager: &'a mut crate::StrokeManager,
}
