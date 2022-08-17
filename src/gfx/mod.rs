pub mod tessellate;
pub mod util;

pub const STANDARD_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

pub struct Gfx {
  wgpu: WgpuCtx,
}

impl Gfx {
  pub async fn init(window: &winit::window::Window) -> Self {
    let wgpu = WgpuCtx::init(window).await;
    Self { wgpu }
  }

  pub fn render<F>(&mut self, render_function: F)
  where
    F: FnOnce(&WgpuCtx, &mut wgpu::CommandEncoder, &wgpu::TextureView),
  {
    let mut encoder = self
      .wgpu
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

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

    render_function(&self.wgpu, &mut encoder, render_target);

    self.wgpu.queue.submit(std::iter::once(encoder.finish()));
    surface_texture.present();
  }

  pub fn resize(&mut self, new_size: [u32; 2]) {
    self.wgpu.resize_surface(new_size);
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
    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(window) };
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
          limits: wgpu::Limits::default(),
        },
        None,
      )
      .await
      .unwrap();

    let window_size = window.inner_size();

    let surface_configuration = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface.get_supported_formats(&adapter)[0],
      width: window_size.width,
      height: window_size.height,
      present_mode: wgpu::PresentMode::Fifo,
    };
    surface.configure(&device, &surface_configuration);

    Self {
      device,
      queue,
      surface,
      surface_configuration,
    }
  }

  pub fn resize_surface(&mut self, new_size: [u32; 2]) {
    self.surface_configuration.width = new_size[0];
    self.surface_configuration.height = new_size[1];
    self
      .surface
      .configure(&self.device, &self.surface_configuration);
  }
}

impl WgpuCtx {
  pub fn device(&self) -> &wgpu::Device {
    &self.device
  }

  pub fn queue(&self) -> &wgpu::Queue {
    &self.queue
  }
}
