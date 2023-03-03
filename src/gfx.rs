pub mod canvas;
pub mod pdf;
pub mod stroke;
pub mod ui;

mod mesh;

use self::{canvas::CanvasRenderer, ui::UiRenderer};

use crate::{pdf::PdfManager, spaces::SpaceManager, stroke::StrokeManager};

use winit::window::Window;

pub struct Gfx {
  wgpu: WgpuCtx,

  ui_renderer: UiRenderer,
  canvas_renderer: CanvasRenderer,
}

impl Gfx {
  pub async fn init(window: &winit::window::Window) -> Self {
    let wgpu = WgpuCtx::init(window).await;

    let ui_renderer = UiRenderer::init(&wgpu.device, wgpu.surface_configuration.format);
    let canvas_renderer = CanvasRenderer::init(&wgpu.device, wgpu.surface_configuration.format);

    Self {
      wgpu,

      ui_renderer,
      canvas_renderer,
    }
  }

  pub fn prepare(
    &mut self,
    window: &Window,
    egui_ctx: &egui::Context,
    egui_shapes: Vec<egui::epaint::ClippedShape>,
    egui_textures_delta: egui::TexturesDelta,

    pdf_manager: Option<&PdfManager>,
    spaces: &SpaceManager,
  ) {
    let mut encoder = self
      .wgpu
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

    self
      .canvas_renderer
      .prepare(&self.wgpu.device, &self.wgpu.queue, spaces, pdf_manager);

    self.ui_renderer.prepare(
      window,
      &self.wgpu.device,
      &self.wgpu.queue,
      &mut encoder,
      egui_ctx,
      egui_shapes,
      egui_textures_delta,
    );

    self.wgpu.queue.submit(std::iter::once(encoder.finish()));
  }
  pub fn render(&mut self, spaces: &SpaceManager, stroke_manager: &StrokeManager) {
    let surface_texture = match self.wgpu.surface.get_current_texture() {
      Ok(frame) => frame,
      Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
        self
          .wgpu
          .surface
          .configure(&self.wgpu.device, &self.wgpu.surface_configuration);
        return;
      }
      Err(wgpu::SurfaceError::Timeout) => {
        tracing::error!("wgpu error: surface texture acquire timeout");
        return;
      }
      Err(wgpu::SurfaceError::OutOfMemory) => panic!("wgpu out of memory"),
    };

    let render_target = &surface_texture
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = self
      .wgpu
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

    {
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("render_pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: render_target,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
            store: true,
          },
          resolve_target: None,
        })],
        depth_stencil_attachment: None,
      });

      self
        .canvas_renderer
        .render(&mut render_pass, spaces, stroke_manager);
      self.ui_renderer.render(&mut render_pass);
    }

    self.wgpu.queue.submit(std::iter::once(encoder.finish()));
    surface_texture.present();
  }

  pub fn resize(&mut self, width: u32, height: u32) {
    self.wgpu.resize_surface(width, height);
  }

  pub fn wgpu(&self) -> &WgpuCtx {
    &self.wgpu
  }
}

pub struct WgpuCtx {
  device: wgpu::Device,
  queue: wgpu::Queue,
  surface: wgpu::Surface,
  surface_configuration: wgpu::SurfaceConfiguration,
}

impl WgpuCtx {
  pub async fn init(window: &winit::window::Window) -> Self {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

    let surface = unsafe { instance.create_surface(window) }.unwrap();

    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
      })
      .await
      .unwrap();

    let (device, queue) = adapter
      .request_device(
        &wgpu::DeviceDescriptor {
          label: None,
          features: wgpu::Features::default(),

          limits: if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults()
          } else {
            wgpu::Limits::default()
          },
        },
        None,
      )
      .await
      .unwrap();

    let [width, height] = {
      let size = window.inner_size();
      [size.width, size.height]
    };

    let surface_configuration = surface.get_default_config(&adapter, width, height).unwrap();
    surface.configure(&device, &surface_configuration);

    Self {
      device,
      queue,
      surface,
      surface_configuration,
    }
  }

  pub fn resize_surface(&mut self, width: u32, height: u32) {
    self.surface_configuration.width = width;
    self.surface_configuration.height = height;
    self
      .surface
      .configure(&self.device, &self.surface_configuration);
  }
}

impl WgpuCtx {
  pub fn device(&self) -> &wgpu::Device {
    &self.device
  }
}

pub struct BufferSized {
  pub buffer: wgpu::Buffer,
  pub size: wgpu::BufferSize,
}
impl BufferSized {
  pub fn new(buffer: wgpu::Buffer, size: wgpu::BufferSize) -> Self {
    Self { buffer, size }
  }
}
