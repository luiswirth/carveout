use crate::{
  content::{command::RemoveStrokesCommand, ContentManager, StrokeId},
  input::InputManager,
  spaces::{Space, SpaceManager},
  stroke::StrokeManager,
};

use parry2d::{
  shape::{Compound, ConvexPolygon, SharedShape},
  transformation::vhacd::{self, VHACD},
};
use std::mem;
use winit::event::MouseButton;

#[derive(Debug, Default)]
pub enum SelectLoop {
  Invalid,
  #[default]
  Inactive,
  Selecting {
    points_screen_logical: Vec<na::Point2<f32>>,
  },
  Selected {
    selected_strokes: Vec<StrokeId>,
  },
}
impl SelectLoop {
  pub fn update(
    &mut self,
    spaces: &SpaceManager,
    input: &InputManager,
    content_manager: &mut ContentManager,
    stroke_manager: &StrokeManager,
  ) {
    if input.got_clicked(MouseButton::Left) {
      match self {
        SelectLoop::Inactive => {
          if let Some(point) = input.curr.cursor_pos_screen_logical {
            let points_screen_logical = vec![point];
            *self = SelectLoop::Selecting {
              points_screen_logical,
            };
          }
        }
        _ => unreachable!(),
      }
    } else if input.is_clicked(MouseButton::Left) {
      match self {
        SelectLoop::Selecting {
          points_screen_logical: screen_points,
        } => {
          if let Some(point) = input.curr.cursor_pos_screen_logical {
            screen_points.push(point);
          }
        }
        _ => unreachable!("{:?}", self),
      }
    } else if input.got_unclicked(MouseButton::Left) {
      match mem::replace(self, Self::Invalid) {
        SelectLoop::Selecting {
          points_screen_logical,
        } => {
          let selected_strokes = Self::get_selection(points_screen_logical, stroke_manager, spaces);
          *self = SelectLoop::Selected { selected_strokes };
        }
        _ => unreachable!("{:?}", self),
      }
    }

    match mem::replace(self, SelectLoop::Invalid) {
      SelectLoop::Selected {
        selected_strokes, ..
      } => {
        content_manager.run_cmd(RemoveStrokesCommand::multiple(selected_strokes));
        *self = SelectLoop::Inactive;
      }
      s => *self = s,
    }
  }

  fn get_selection(
    points_screen_logical: Vec<na::Point2<f32>>,
    stroke_manager: &StrokeManager,
    spaces: &SpaceManager,
  ) -> Vec<StrokeId> {
    let isometry = na::Isometry2::default();

    let canvas_points: Vec<_> = points_screen_logical
      .iter()
      .map(|p| spaces.transform_point(*p, Space::ScreenLogical, Space::Canvas))
      .collect();
    let params = vhacd::VHACDParameters::default();
    let len = points_screen_logical.len() as u32;
    let indices: Vec<_> = (0..len - 1)
      .map(|i| [i, i + 1])
      .chain(std::iter::once([len - 1, 0]))
      .collect();
    let vhacd = VHACD::decompose(&params, &canvas_points, &indices, false);
    let convex_hulls = vhacd.compute_convex_hulls(1);
    let convex_polygons = convex_hulls
      .into_iter()
      .map(|h| ConvexPolygon::from_convex_polyline(h).unwrap());
    let shapes = convex_polygons
      .into_iter()
      .map(|p| (isometry, SharedShape::new(p)))
      .collect();
    let compound = Compound::new(shapes);

    stroke_manager
      .data()
      .parry_meshes
      .iter()
      .filter(|(_, stroke_mesh)| {
        parry2d::query::intersection_test(&isometry, &compound, &isometry, *stroke_mesh).unwrap()
      })
      .map(|(id, _)| *id)
      .collect()
  }
}
