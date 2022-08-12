use super::{OngoingStroke, Stroke};

use crate::{
  canvas::{tool::PenConfig, Camera},
  util::space::*,
};

use replace_with::replace_with_or_default;
use winit::{
  event::{self, WindowEvent},
  window::Window,
};

const SAMPLE_DISTANCE_TOLERANCE: ScreenPixelUnit = ScreenPixelUnit::new(1.0);

#[derive(Default)]
pub struct SampledStroke {
  pub samples: Vec<InteractionSample>,
}

pub struct InteractionSample {
  pub pos: ScreenPixelPoint,
  pub force: Option<f32>,
}

pub fn handle_event(
  event: &crate::Event,
  window: &Window,
  camera: &Camera,
  pen_config: &PenConfig,
  stroke: &mut OngoingStroke,
) {
  if let event::Event::WindowEvent {
    window_id: _,
    event,
  } = event
  {
    match event {
      WindowEvent::Touch(touch) => {
        let pos = WindowPhysicalPoint::from_underlying(touch.location);
        let pos = WindowLogicalPoint::from_physical(pos, window.scale_factor() as f32);
        let pos = ScreenPixelPoint::try_from_window_logical(pos, camera);
        if let Some(pos) = pos {
          try_record_sample(
            InteractionSample {
              pos,
              force: touch.force.map(|force| force.normalized() as f32),
            },
            stroke,
          );
        }
      }
      WindowEvent::CursorMoved { position, .. } => {
        let pos = WindowPhysicalPoint::from_underlying(*position);
        let pos = WindowLogicalPoint::from_physical(pos, window.scale_factor() as f32);
        let pos = ScreenPixelPoint::try_from_window_logical(pos, camera);
        if let Some(pos) = pos {
          try_record_sample(InteractionSample { pos, force: None }, stroke);
        }
      }
      WindowEvent::MouseInput { state, button, .. } => {
        if *button == event::MouseButton::Left {
          match state {
            event::ElementState::Pressed => begin_record(stroke, pen_config),
            event::ElementState::Released => end_record(stroke),
          }
        }
      }
      _ => {}
    }
  }
}

fn begin_record(stroke: &mut OngoingStroke, pen_config: &PenConfig) {
  match stroke {
    OngoingStroke::None => {
      *stroke = OngoingStroke::Ongoing(Stroke::new(pen_config.color, pen_config.width))
    }
    OngoingStroke::Ongoing(_) => {
      panic!("Started recording a new stroke, although there already is one.")
    }
    OngoingStroke::Finished(_) => {
      panic!("Started recording a new stroke, although the finished one is still here.")
    }
  }
}

fn end_record(stroke: &mut OngoingStroke) {
  replace_with_or_default(stroke, |c| match c {
    OngoingStroke::Ongoing(c) => OngoingStroke::Finished(c),
    OngoingStroke::None => panic!("There is no stroke to stop recording."),
    OngoingStroke::Finished(_) => panic!("The stroke recording was already stopped."),
  })
}

fn try_record_sample(new_sample: InteractionSample, stroke: &mut OngoingStroke) {
  if let OngoingStroke::Ongoing(c) = stroke {
    match c.sampled.samples.last() {
      None => c.sampled.samples.push(new_sample),
      Some(last_sample) => {
        let square_dist = (new_sample.pos - last_sample.pos).magnitude_squared();
        if square_dist > SAMPLE_DISTANCE_TOLERANCE * SAMPLE_DISTANCE_TOLERANCE {
          c.sampled.samples.push(new_sample);
        }
      }
    }
  }
}
