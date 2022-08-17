use crate::canvas::{
  content::{AddStrokeCommand, CanvasContent},
  space::*,
  stroke::{InteractionSample, Stroke},
  tool::PenConfig,
  undo::UndoTree,
  CameraWithScreen,
};

use winit::{
  event::{self, WindowEvent},
  window::Window,
};

const SAMPLE_DISTANCE_TOLERANCE: ScreenPixelUnit = ScreenPixelUnit::new(1.0);

#[derive(Default)]
pub struct PenInputHandler {
  clicked: bool,
}

impl PenInputHandler {
  pub fn handle_event(
    &mut self,
    event: &WindowEvent,
    window: &Window,
    camera_screen: &CameraWithScreen,
    pen_config: &PenConfig,
    undo_tree: &mut UndoTree,
    content: &mut CanvasContent,
  ) {
    let stroke = &mut content.ongoing().stroke;
    match event {
      WindowEvent::CursorMoved { position, .. } => {
        let pos = position.to_logical(window.scale_factor());
        let pos = ScreenPixelPoint::try_from_window_logical(pos, camera_screen);
        if self.clicked {
          if let Some(pos) = pos {
            self.record_sample(InteractionSample { pos }, stroke, pen_config);
          }
        }
      }
      WindowEvent::MouseInput { state, button, .. } => {
        if *button == event::MouseButton::Left {
          match state {
            event::ElementState::Pressed => self.clicked = true,
            event::ElementState::Released => {
              self.clicked = false;

              // TODO: clean up
              if let Some(s) = content.ongoing().stroke.take() {
                undo_tree.do_it(Box::new(AddStrokeCommand::new(s)), content.persistent());
              }
            }
          }
        }
      }
      _ => {}
    }
  }

  fn record_sample(
    &self,
    new_sample: InteractionSample,
    stroke: &mut Option<Stroke>,
    pen_config: &PenConfig,
  ) {
    let stroke = stroke.get_or_insert(Stroke::new(pen_config.color, pen_config.width));
    match stroke.sampled.samples.last() {
      None => stroke.sampled.samples.push(new_sample),
      Some(last_sample) => {
        let square_dist = (new_sample.pos - last_sample.pos).magnitude_squared();
        if square_dist > SAMPLE_DISTANCE_TOLERANCE * SAMPLE_DISTANCE_TOLERANCE {
          stroke.sampled.samples.push(new_sample);
        }
      }
    }
  }
}
