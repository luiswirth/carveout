use super::state::InputState;

use crate::canvas::{
  content::{command::AddStrokeCommand, ContentManager, StrokeId},
  gfx::CameraWithScreen,
  space::*,
  stroke::Stroke,
  tool::PenConfig,
};

use winit::event::MouseButton;

const TOLERANCE: ScreenPixelUnit = ScreenPixelUnit::new(1.0);

#[derive(Default)]
pub struct PenInputHandler {
  prev_point: Option<CanvasPoint>,
  stroke: Option<StrokeId>,
}

impl PenInputHandler {
  pub fn update(
    &mut self,
    input: &InputState,
    content: &mut ContentManager,
    pen_config: &PenConfig,
    camera_screen: &CameraWithScreen,
  ) {
    if input.got_unclicked(MouseButton::Left) {
      self.finish_stroke()
    }
    if input.is_clicked(MouseButton::Left) {
      self.sample_stroke(input, content, pen_config, camera_screen)
    }
  }

  fn sample_stroke(
    &mut self,
    input: &InputState,
    content: &mut ContentManager,
    pen_config: &PenConfig,
    camera_screen: &CameraWithScreen,
  ) {
    if let Some(curr_point) = input.curr.cursor_pos.as_ref().map(|c| c.canvas) {
      if let Some(prev_point) = self.prev_point {
        let diff = curr_point - prev_point;
        let diff = ScreenPixelVector::from_canvas(diff, camera_screen);
        let dist = diff.magnitude_squared();
        if dist > TOLERANCE * TOLERANCE {
          match self.stroke {
            None => {
              let points = vec![prev_point, curr_point];
              let stroke = Stroke::new(points, pen_config.color, pen_config.width);
              content.run_cmd(AddStrokeCommand::new(stroke));

              let stroke = content.delta().strokes.added.last().unwrap();
              self.stroke = Some(*stroke);
            }
            Some(stroke) => {
              let mut access_mut = content.access_mut();
              let stroke = access_mut.modify_stroke(stroke);
              stroke.add_point(curr_point);
            }
          }
          self.prev_point = Some(curr_point);
        }
      } else {
        self.prev_point = Some(curr_point);
      }
    }
  }

  fn finish_stroke(&mut self) {
    self.prev_point = None;
    self.stroke = None;
  }
}
