use super::{
  stroke::{Stroke, StrokeId},
  undo::Command,
};

pub struct CanvasContent {
  ongoing: OngoingContent,
  persistent: PersistentContent,
}
impl CanvasContent {
  pub fn init() -> Self {
    let ongoing = OngoingContent::init();
    let persistent = PersistentContent::init();
    Self {
      ongoing,
      persistent,
    }
  }

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
pub struct OngoingContent {
  pub stroke: Option<Stroke>,
}
impl OngoingContent {
  fn init() -> Self {
    let stroke = None;
    Self { stroke }
  }
}

/// Should only be mutated through `Command`s.
pub struct PersistentContent {
  strokes: Vec<Stroke>,
}
impl PersistentContent {
  fn init() -> Self {
    let strokes = Vec::new();
    Self { strokes }
  }

  pub fn strokes(&self) -> &[Stroke] {
    &self.strokes
  }

  // TODO: ILLEGAL! no mutable access should be granted
  pub fn strokes_mut(&mut self) -> &mut [Stroke] {
    &mut self.strokes
  }
}

pub struct AddStrokeCommand(Option<Stroke>);
impl AddStrokeCommand {
  pub fn new(stroke: Stroke) -> Self {
    Self(Some(stroke))
  }
}
impl Command for AddStrokeCommand {
  fn execute(&mut self, content: &mut PersistentContent) {
    content.strokes.push(self.0.take().unwrap());
  }

  fn rollback(&mut self, content: &mut PersistentContent) {
    // TODO: fix
    self.0 = Some(content.strokes.pop().unwrap());
  }
}

pub enum RemoveStrokeCommand {
  Before(StrokeId),
  After(Box<Stroke>),
}
impl RemoveStrokeCommand {
  pub fn new(id: StrokeId) -> Self {
    Self::Before(id)
  }
}
impl Command for RemoveStrokeCommand {
  fn execute(&mut self, content: &mut PersistentContent) {
    // TODO: optimize! don't look through all strokes. avoid O(n)
    match self {
      Self::Before(id) => {
        let stroke = content
          .strokes
          .remove(content.strokes.iter().position(|s| s.id == *id).unwrap());
        *self = Self::After(Box::new(stroke));
      }
      Self::After(_) => unreachable!(),
    }
  }

  fn rollback(&mut self, content: &mut PersistentContent) {
    let id;
    match std::mem::replace(self, Self::Before(StrokeId::nil())) {
      Self::After(stroke) => {
        id = stroke.id;
        content.strokes.push(*stroke);
      }
      Self::Before(_) => unreachable!(),
    }
    *self = Self::Before(id);
  }
}
