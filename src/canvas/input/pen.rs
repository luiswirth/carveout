use crate::canvas::{
  content::{AddStrokeCommand, CanvasContent},
  space::*,
  stroke::Stroke,
  tool::PenConfig,
  undo::ContentCommander,
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

  points: Vec<ScreenPixelPoint>,
}

impl PenInputHandler {
  pub fn handle_event(
    &mut self,
    event: &WindowEvent,
    window: &Window,
    camera_screen: &CameraWithScreen,
    pen_config: &PenConfig,
    content_commander: &mut ContentCommander,
    content: &mut CanvasContent,
  ) {
    match event {
      WindowEvent::CursorMoved { position, .. } => {
        let pos = position.to_logical(window.scale_factor());
        let pos = ScreenPixelPoint::try_from_window_logical(pos, camera_screen);
        if self.clicked {
          if let Some(pos) = pos {
            self.record_sampled_point(
              pos,
              &mut content.ongoing().stroke,
              camera_screen,
              pen_config,
            );
          }
        }
      }
      WindowEvent::MouseInput { state, button, .. } => {
        if *button == event::MouseButton::Left {
          match state {
            event::ElementState::Pressed => {
              self.clicked = true;
              self.start_stroke();
            }
            event::ElementState::Released => {
              self.clicked = false;
              self.finish_stroke(content_commander, content);
            }
          }
        }
      }
      _ => {}
    }
  }

  fn start_stroke(&mut self) {
    self.points = Vec::new();
  }

  fn record_sampled_point(
    &mut self,
    new_point: ScreenPixelPoint,
    ongoing_stroke: &mut Option<Stroke>,
    camera_screen: &CameraWithScreen,
    pen_config: &PenConfig,
  ) {
    match self.points.last() {
      None => {
        self.points.push(new_point);
      }
      Some(last_point) => {
        let square_dist = (new_point - last_point).magnitude_squared();
        if square_dist > SAMPLE_DISTANCE_TOLERANCE * SAMPLE_DISTANCE_TOLERANCE {
          self.points.push(new_point);
          match ongoing_stroke {
            None => {
              let points = self
                .points
                .iter()
                .map(|p| CanvasPointExt::from_screen(*p, camera_screen))
                .collect();
              *ongoing_stroke = Some(Stroke::new(points, pen_config.color, pen_config.width))
            }
            Some(ongoing_stroke) => {
              self.points.push(new_point);
              ongoing_stroke.add_point(CanvasPointExt::from_screen(new_point, camera_screen));
            }
          }
        }
      }
    }
  }

  fn finish_stroke(
    &mut self,
    content_commander: &mut ContentCommander,
    content: &mut CanvasContent,
  ) {
    if let Some(finished_stroke) = content.ongoing().stroke.take() {
      content_commander.do_it(Box::new(AddStrokeCommand::new(finished_stroke)));
    }
  }
}
