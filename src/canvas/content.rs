use std::collections::HashMap;

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
  strokes: HashMap<StrokeId, Stroke>,
}
impl PersistentContent {
  pub fn strokes(&self) -> &HashMap<StrokeId, Stroke> {
    &self.strokes
  }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AddStrokeCommand {
  Invalid,
  Before(Box<Stroke>),
  After(StrokeId),
}

impl AddStrokeCommand {
  pub fn new(stroke: Stroke) -> Self {
    Self::Before(Box::new(stroke))
  }
}
#[typetag::serde]
impl Command for AddStrokeCommand {
  fn execute(&mut self, content: &mut PersistentContent) -> Result<(), ()> {
    match std::mem::replace(self, Self::Invalid) {
      Self::Before(stroke) => {
        let id = stroke.id();
        let result = content.strokes.insert(id, *stroke);
        assert!(result.is_none());
        *self = Self::After(id);
      }
      _ => unreachable!(),
    };
    Ok(())
  }

  fn rollback(&mut self, content: &mut PersistentContent) {
    match std::mem::replace(self, Self::Invalid) {
      Self::After(id) => {
        let stroke = content.strokes.remove(&id).unwrap();
        *self = Self::Before(Box::new(stroke));
      }
      _ => unreachable!(),
    };
  }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum RemoveStrokeCommand {
  Invalid,
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
  fn execute(&mut self, content: &mut PersistentContent) -> Result<(), ()> {
    match self {
      Self::Before(id) => {
        let stroke = content.strokes.remove(id).unwrap();
        *self = Self::After(Box::new(stroke));
        Ok(())
      }
      _ => unreachable!(),
    }
  }

  fn rollback(&mut self, content: &mut PersistentContent) {
    match std::mem::replace(self, Self::Invalid) {
      Self::After(stroke) => {
        let id = stroke.id();
        let result = content.strokes.insert(id, *stroke);
        assert!(result.is_none());
        *self = Self::Before(id);
      }
      _ => unreachable!(),
    }
  }
}
