mod path;
mod render;
mod sample;
mod tessellate;

pub use self::{render::StrokeRenderer, tessellate::StrokeTessellator};

pub use self::{path::PathStroke, sample::SampledStroke, tessellate::TessallatedStroke};

use palette::LinSrgb;
use replace_with::replace_with_or_default;

use super::{tool::PenConfig, CanvasPortal};

pub struct StrokeManager {
  ongoing_stroke: OngoingStroke,
  finished_strokes: Vec<Stroke>,
  tessellator: StrokeTessellator,
  renderer: StrokeRenderer,
}

impl StrokeManager {
  pub fn init(device: &wgpu::Device) -> Self {
    let ongoing_stroke = OngoingStroke::default();
    let finished_strokes = Vec::new();
    let tessellator = StrokeTessellator::init();
    let renderer = StrokeRenderer::init(device);

    Self {
      ongoing_stroke,
      finished_strokes,

      tessellator,
      renderer,
    }
  }

  pub fn handle_event(
    &mut self,
    event: &crate::Event,
    window: &winit::window::Window,
    portal: &CanvasPortal,
    pen_config: &PenConfig,
  ) {
    sample::handle_event(event, window, portal, pen_config, &mut self.ongoing_stroke);
  }

  pub fn render(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    portal: &CanvasPortal,
  ) {
    replace_with_or_default(&mut self.ongoing_stroke, |c| match c {
      OngoingStroke::Ongoing(mut c) => {
        c.path = PathStroke::new(&c.sampled, portal);
        c.tessellated = self.tessellator.tessellate(&c.path, &c.shared_info);
        OngoingStroke::Ongoing(c)
      }
      OngoingStroke::Finished(c) => {
        self.finished_strokes.push(c);
        OngoingStroke::None
      }
      OngoingStroke::None => OngoingStroke::None,
    });

    self.renderer.render(
      device,
      queue,
      encoder,
      portal,
      self
        .finished_strokes
        .iter_mut()
        .chain(self.ongoing_stroke.as_ongoing_mut())
        .map(|c| &mut c.tessellated),
    );
  }
}

/// bool indicates if finished
#[derive(Default)]
pub enum OngoingStroke {
  #[default]
  None,
  Ongoing(Stroke),
  Finished(Stroke),
}
impl OngoingStroke {
  pub fn as_ongoing_mut(&mut self) -> Option<&mut Stroke> {
    match self {
      Self::Ongoing(c) => Some(c),
      _ => None,
    }
  }
}

#[derive(Default)]
pub struct Stroke {
  pub sampled: SampledStroke,
  pub path: PathStroke,
  pub tessellated: TessallatedStroke,

  pub shared_info: SharedStrokeInfo,
}
impl Stroke {
  pub fn new(color: LinSrgb, width: f32) -> Self {
    Self {
      shared_info: SharedStrokeInfo { width, color },
      ..Default::default()
    }
  }
}

pub struct SharedStrokeInfo {
  pub width: f32,
  pub color: LinSrgb,
}

impl Default for SharedStrokeInfo {
  fn default() -> Self {
    Self {
      color: palette::named::WHITE.into_format().into_linear(),
      width: 1.0,
    }
  }
}
