use crate::{
  content::{command::RemoveStrokesCommand, ContentManager, StrokeId},
  input::InputManager,
  spaces::{Space, SpaceManager},
  stroke::StrokeManager,
};

use parry2d::query::PointQuery;

pub fn update_eraser(
  input: &InputManager,
  content_manager: &mut ContentManager,
  stroke_manager: &StrokeManager,
  spaces: &SpaceManager,
) {
  if !input.is_clicked(winit::event::MouseButton::Left) {
    return;
  }
  if let Some(pos_screen_logical) = input.curr.cursor_pos_screen_logical {
    let pos_canvas =
      spaces.transform_point(pos_screen_logical, Space::ScreenLogical, Space::Canvas);
    let stroke_data = stroke_manager.data();
    // TODO: stop iterating through all strokes. Use spatial partitioning.
    let remove_list: Vec<StrokeId> = content_manager
      .access()
      .strokes()
      .map(|(id, _)| id)
      .filter(|id| {
        let mesh = stroke_data.parry_meshes.get(id).expect("No parry data.");
        mesh.contains_point(&na::Isometry2::default(), &pos_canvas)
      })
      .collect();

    for id in remove_list {
      content_manager.run_cmd(RemoveStrokesCommand::single(id))
    }
  }
}
