use crate::{
  content::{command::AddStrokeCommand, ContentManager, StrokeId},
  input::InputManager,
  spaces::*,
  stroke::Stroke,
  tools::PenConfig,
};

use winit::event::MouseButton;

#[derive(Default)]
pub struct Pen {
  prev_point_canvas: Option<na::Point2<f32>>,
  stroke: Option<StrokeId>,
}

impl Pen {
  pub fn update(
    &mut self,
    input: &InputManager,
    content_manager: &mut ContentManager,
    pen_config: &PenConfig,
    spaces: &SpaceManager,
  ) {
    if input.got_unclicked(MouseButton::Left) {
      self.finish_stroke()
    }
    if input.is_clicked(MouseButton::Left) {
      self.sample_stroke(input, content_manager, pen_config, spaces)
    }
  }

  fn sample_stroke(
    &mut self,
    input: &InputManager,
    content_manager: &mut ContentManager,
    pen_config: &PenConfig,
    spaces: &SpaceManager,
  ) {
    if let Some(curr_point_screen_logical) = input.curr.cursor_pos_screen_logical.to_owned() {
      let curr_point_canvas = spaces.transform_point(
        curr_point_screen_logical,
        Space::ScreenLogical,
        Space::Canvas,
      );
      if let Some(prev_point_canvas) = self.prev_point_canvas {
        let prev_point_screen_logical =
          spaces.transform_point(prev_point_canvas, Space::Canvas, Space::ScreenLogical);
        let diff_logical = curr_point_screen_logical - prev_point_screen_logical;
        let dist_logical = diff_logical.magnitude_squared();
        let tolerance_logical_sqr = 1.0;
        if dist_logical > tolerance_logical_sqr {
          match self.stroke {
            None => {
              let points = vec![prev_point_canvas, curr_point_canvas];
              let stroke = Stroke::new(points, pen_config.color, pen_config.width);
              content_manager.run_cmd(AddStrokeCommand::new(stroke));

              let stroke = content_manager.delta().strokes.added.last().unwrap();
              self.stroke = Some(*stroke);
            }
            Some(stroke) => {
              let mut access_mut = content_manager.access_mut();
              let stroke = access_mut.modify_stroke(stroke);
              stroke.add_point(curr_point_canvas);
            }
          }
          self.prev_point_canvas = Some(curr_point_canvas);
        }
      } else {
        self.prev_point_canvas = Some(curr_point_canvas);
      }
    }
  }

  fn finish_stroke(&mut self) {
    self.prev_point_canvas = None;
    self.stroke = None;
  }
}
