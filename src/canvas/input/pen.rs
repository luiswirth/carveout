use crate::canvas::{
  content::{command::AddStrokeCommand, ContentManager, StrokeId},
  space::*,
  stroke::Stroke,
  tool::PenConfig,
  CameraWithScreen,
};

use winit::{
  event::{self, WindowEvent},
  window::Window,
};

const SAMPLE_DISTANCE_TOLERANCE: ScreenPixelUnit = ScreenPixelUnit::new(1.0);

#[derive(Default)]
pub struct PenInputHandler {
  is_writing: bool,

  stroke: Option<StrokeId>,
  points: Vec<ScreenPixelPoint>,
}

impl PenInputHandler {
  pub fn handle_event(
    &mut self,
    event: &WindowEvent,
    window: &Window,
    content: &mut ContentManager,
    camera_screen: &CameraWithScreen,
    pen_config: &PenConfig,
  ) {
    match event {
      WindowEvent::CursorMoved { position, .. } => {
        let pos = position.to_logical(window.scale_factor());
        let pos = ScreenPixelPoint::try_from_window_logical(pos, camera_screen);

        if self.is_writing {
          if let Some(pos) = pos {
            self.record_sampled_point(pos, content, camera_screen, pen_config);
          }
        }
      }
      WindowEvent::MouseInput { state, button, .. } => {
        if *button == event::MouseButton::Left {
          match state {
            event::ElementState::Pressed => self.start_stroke(),
            event::ElementState::Released => self.finish_stroke(),
          }
        }
      }
      _ => {}
    }
  }

  fn start_stroke(&mut self) {
    self.is_writing = true;
  }

  fn record_sampled_point(
    &mut self,
    new_point: ScreenPixelPoint,
    content: &mut ContentManager,
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
          match self.stroke {
            None => {
              let points = self
                .points
                .iter()
                .map(|p| CanvasPointExt::from_screen(*p, camera_screen))
                .collect();

              let stroke = Stroke::new(points, pen_config.color, pen_config.width);
              content.run_cmd(AddStrokeCommand::new(stroke));

              // TODO: find a nicer way to do this.
              let stroke = content.delta().strokes.added.last().unwrap();
              self.stroke = Some(*stroke);
            }
            Some(stroke) => {
              self.points.push(new_point);
              let mut access_mut = content.access_mut();
              let stroke = access_mut.modify_stroke(stroke);
              stroke.add_point(CanvasPointExt::from_screen(new_point, camera_screen));
            }
          }
        }
      }
    }
  }

  fn finish_stroke(&mut self) {
    self.is_writing = false;

    self.stroke = None;
    self.points.clear();
  }
}
