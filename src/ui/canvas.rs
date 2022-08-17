use std::{cell::RefCell, rc::Rc};

use super::render_target::UiRenderTarget;

pub struct CanvasUi {
  screen: Rc<RefCell<CanvasScreen>>,
  has_focus: bool,
}

impl CanvasUi {
  pub fn init(device: &wgpu::Device, ui_renderer: &mut super::backend::Renderer) -> Self {
    let render_target = UiRenderTarget::new(device, ui_renderer);
    let rect = egui::Rect::NAN;

    let screen = CanvasScreen {
      render_target,
      rect,
    };
    let screen = Rc::new(RefCell::new(screen));
    let has_focus = false;
    Self { screen, has_focus }
  }

  pub fn ui(
    &mut self,
    ctx: &egui::Context,
    device: &wgpu::Device,
    ui_renderer: &mut super::backend::Renderer,
  ) {
    egui::CentralPanel::default().show(ctx, |ui| {
      let mut screen = self.screen.borrow_mut();
      let response = screen
        .render_target
        .ui(ui, device, ui_renderer, ui.available_size());
      screen.rect = response.rect;
      self.has_focus = response.hovered();
    });
  }

  pub fn screen(&self) -> &Rc<RefCell<CanvasScreen>> {
    &self.screen
  }

  pub fn has_focus(&self) -> bool {
    self.has_focus
  }
}

pub struct CanvasScreen {
  render_target: UiRenderTarget,
  rect: egui::Rect,
}
impl CanvasScreen {
  pub fn rect(&self) -> egui::Rect {
    self.rect
  }

  pub fn render_target(&self) -> &wgpu::TextureView {
    self.render_target.view()
  }
}
