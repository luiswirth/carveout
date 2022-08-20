use serde::{Deserialize, Serialize};

use super::{
  protocol::Command,
  stroke::{Stroke, StrokeId},
};

#[derive(Default)]
pub struct CanvasContent {
  ongoing: OngoingContent,
  persistent: PersistentContent,
}
impl CanvasContent {
  pub fn ongoing(&mut self) -> &mut OngoingContent {
    &mut self.ongoing
  }

  pub fn persistent(&self) -> &PersistentContent {
    &self.persistent
  }

  pub fn persistent_mut(&mut self) -> &mut PersistentContent {
    &mut self.persistent
  }

  pub fn ongoing_persistent_mut(&mut self) -> (&mut OngoingContent, &mut PersistentContent) {
    (&mut self.ongoing, &mut self.persistent)
  }
}

/// Can be freely mutated.
#[derive(Default)]
pub struct OngoingContent {
  pub stroke: Option<Stroke>,
}

/// Should only be mutated through `Command`s.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct PersistentContent {
  strokes: Vec<Stroke>,
}
impl PersistentContent {
  pub fn strokes(&self) -> &[Stroke] {
    &self.strokes
  }
}

#[derive(Serialize, Deserialize)]
pub struct AddStrokeCommand(Option<Stroke>);
impl AddStrokeCommand {
  pub fn new(stroke: Stroke) -> Self {
    Self(Some(stroke))
  }
}
#[typetag::serde]
impl Command for AddStrokeCommand {
  fn execute(&mut self, content: &mut PersistentContent) {
    content.strokes.push(self.0.take().unwrap());
  }

  fn rollback(&mut self, content: &mut PersistentContent) {
    // TODO: fix
    self.0 = Some(content.strokes.pop().unwrap());
  }
}

#[derive(Serialize, Deserialize)]
pub enum RemoveStrokeCommand {
  Before(StrokeId),
  After(Box<Stroke>),
}
impl RemoveStrokeCommand {
  pub fn new(id: StrokeId) -> Self {
    Self::Before(id)
  }
}
#[typetag::serde]
impl Command for RemoveStrokeCommand {
  fn execute(&mut self, content: &mut PersistentContent) {
    // TODO: optimize! don't look through all strokes. avoid O(n)
    match self {
      Self::Before(id) => {
        let stroke = content
          .strokes
          .remove(content.strokes.iter().position(|s| s.id() == *id).unwrap());
        *self = Self::After(Box::new(stroke));
      }
      Self::After(_) => unreachable!(),
    }
  }

  fn rollback(&mut self, content: &mut PersistentContent) {
    let id;
    match std::mem::replace(self, Self::Before(StrokeId::nil())) {
      Self::After(stroke) => {
        id = stroke.id();
        content.strokes.push(*stroke);
      }
      Self::Before(_) => unreachable!(),
    }
    *self = Self::Before(id);
  }
}
