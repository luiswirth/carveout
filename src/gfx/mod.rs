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

  pub fn handle_event(&mut self, event: &crate::Event<'_>) {
    use winit::event::WindowEvent;

    if let crate::Event::WindowEvent {
      window_id: _,
      event,
    } = event
    {
      match event {
        WindowEvent::Resized(new_size) => self.resize_surface([new_size.width, new_size.height]),
        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
          self.resize_surface([new_inner_size.width, new_inner_size.height]);
        }
        _ => {}
      }
    }
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
      .create_view(&wgpu::TextureViewDescriptor {
        label: Some("surface_render_target_view"),
        format: None,
        dimension: None,
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
      });

    render_function(&self.wgpu, &mut encoder, render_target);

    self.wgpu.queue.submit(std::iter::once(encoder.finish()));
    surface_texture.present();
  }

  fn resize_surface(&mut self, new_size: [u32; 2]) {
    self.wgpu.resize_surface(new_size);
  }

  /// Get a reference to the graphics's wgpu.
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
    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
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
      format: STANDARD_TEXTURE_FORMAT,
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
