use parry2d::query::PointQuery;

use crate::canvas::{
  content::{command::RemoveStrokeCommand, ContentManager, StrokeId},
  stroke::StrokeManager,
};

use super::state::InputState;

pub fn update_eraser(
  input: &InputState,
  content: &mut ContentManager,
  stroke_manager: &StrokeManager,
) {
  if !input.is_clicked(winit::event::MouseButton::Left) {
    return;
  }
  if let Some(pos) = input.curr.cursor_pos.as_ref().map(|c| c.canvas) {
    // TODO: stop iterating through all strokes. Use spatial partitioning.
    let remove_list: Vec<StrokeId> = content
      .access()
      .strokes()
      .map(|(id, _)| id)
      .filter(|id| {
        let mesh = stroke_manager
          .data()
          .parry_meshes
          .get(id)
          .expect("No parry data.");
        mesh.contains_point(&na::Isometry2::default(), &pos.cast())
      })
      .collect();

    for id in remove_list {
      content.run_cmd(RemoveStrokeCommand::new(id))
    }
  }
}
