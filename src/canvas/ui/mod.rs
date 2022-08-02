mod render_target;

use render_target::UiRenderTarget;

use crate::util::space::{SpaceUnderlyingConversion, WindowLogicalBox};

use crate::ui::Renderer as UiRenderer;

pub struct CanvasUi {
  render_target: UiRenderTarget,
  ui_box: WindowLogicalBox,
}

impl CanvasUi {
  pub fn init(device: &wgpu::Device, ui_renderer: &mut UiRenderer) -> Self {
    let render_target = UiRenderTarget::new(device, ui_renderer);
    let ui_box = WindowLogicalBox::default();

    Self {
      render_target,
      ui_box,
    }
  }

  pub fn build_ui(
    &mut self,
    ctx: &egui::Context,
    device: &wgpu::Device,
    ui_renderer: &mut UiRenderer,
  ) {
    egui::CentralPanel::default().show(ctx, |ui| {
      let response = self
        .render_target
        .ui(ui, device, ui_renderer, ui.available_size());
      self.ui_box = WindowLogicalBox::from_underlying(response.rect);
    });
  }

  pub fn render_target(&self) -> &wgpu::TextureView {
    self.render_target.view()
  }

  pub fn ui_box(&self) -> WindowLogicalBox {
    self.ui_box
  }
}
