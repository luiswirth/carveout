use crate::ui::Renderer as UiRenderer;

/// A target for rendering in the UI.
/// Uses all available space.
pub struct UiRenderTarget {
  texture_descriptor: wgpu::TextureDescriptor<'static>,
  texture: wgpu::Texture,
  view: wgpu::TextureView,
  ui_texture_id: egui::TextureId,
}

impl UiRenderTarget {
  pub fn new(device: &wgpu::Device, renderer: &mut UiRenderer) -> Self {
    let texture_descriptor = wgpu::TextureDescriptor {
      label: Some("ui_render_target_texture"),
      size: wgpu::Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: crate::gfx::STANDARD_TEXTURE_FORMAT,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
    };

    let texture = device.create_texture(&texture_descriptor);
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let ui_texture_id = renderer.register_native_texture(device, &view, wgpu::FilterMode::Linear);

    Self {
      texture,
      texture_descriptor,
      view,
      ui_texture_id,
    }
  }

  pub fn ui(
    &mut self,
    ui: &mut egui::Ui,
    device: &wgpu::Device,
    ui_renderer: &mut UiRenderer,
    logical_size: egui::Vec2,
  ) -> egui::Response {
    let physical_size = logical_size * ui.input().pixels_per_point();
    self.check_resize(device, ui_renderer, physical_size);
    ui.add(egui::Image::new(self.ui_texture_id, logical_size).sense(egui::Sense::hover()))
  }

  pub fn view(&self) -> &wgpu::TextureView {
    &self.view
  }
}

impl UiRenderTarget {
  fn check_resize(
    &mut self,
    device: &wgpu::Device,
    ui_renderer: &mut UiRenderer,
    physical_size: egui::Vec2,
  ) {
    // check if resize necessary
    let new_width = physical_size.x as u32;
    let new_height = physical_size.y as u32;
    let [curr_width, curr_height] = {
      let curr_size = &mut self.texture_descriptor.size;
      [&mut curr_size.width, &mut curr_size.height]
    };
    let same_size = *curr_width == new_width && *curr_height == new_height;
    let no_size = new_width == 0 || new_height == 0;
    if same_size || no_size {
      return;
    }

    // update descriptor
    *curr_width = new_width;
    *curr_height = new_height;

    // destroy old
    ui_renderer.free_texture(&self.ui_texture_id);

    // create new
    self.texture = device.create_texture(&self.texture_descriptor);
    self.view = self
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());
    self.ui_texture_id =
      ui_renderer.register_native_texture(device, &self.view, wgpu::FilterMode::Linear);
  }
}
