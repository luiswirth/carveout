use super::{stroke::Stroke, undo::Command};

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

  pub fn persistent(&mut self) -> &mut PersistentContent {
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
    self.0 = Some(content.strokes.pop().unwrap());
  }
}
