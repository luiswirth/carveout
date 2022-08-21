use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};

use crate::canvas::stroke::{Stroke, StrokeId};

use super::PersistentContent;

#[typetag::serde(tag = "type")]
pub trait ProtocolCommand: DynClone {
  fn execute(&mut self, content: &mut PersistentContent) -> Result<(), ()>;
  fn rollback(&mut self, content: &mut PersistentContent);
}
dyn_clone::clone_trait_object!(ProtocolCommand);

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
impl ProtocolCommand for AddStrokeCommand {
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
impl ProtocolCommand for RemoveStrokeCommand {
  fn execute(&mut self, content: &mut PersistentContent) -> Result<(), ()> {
    match self {
      Self::Before(id) => {
        let stroke = content.strokes.remove(id);
        match stroke {
          Some(stroke) => {
            *self = Self::After(Box::new(stroke));
            Ok(())
          }
          None => Err(()),
        }
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
