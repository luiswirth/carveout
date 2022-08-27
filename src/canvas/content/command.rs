use super::{access::ContentAccessMut, StrokeId};

use crate::canvas::stroke::Stroke;

use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};

#[typetag::serde(tag = "type", content = "value")]
pub trait ProtocolCommand: DynClone {
  fn execute(&mut self, content: ContentAccessMut);
  fn rollback(&mut self, content: ContentAccessMut);
}
dyn_clone::clone_trait_object!(ProtocolCommand);

#[derive(Clone, Serialize, Deserialize)]
pub enum AddStrokeCommand {
  Invalid,
  Before(Box<Stroke>),
  After(StrokeId),
}

impl AddStrokeCommand {
  pub fn new(stroke: Stroke) -> Box<Self> {
    Box::new(Self::Before(Box::new(stroke)))
  }
}
#[typetag::serde]
impl ProtocolCommand for AddStrokeCommand {
  fn execute(&mut self, mut content: ContentAccessMut) {
    match std::mem::replace(self, Self::Invalid) {
      Self::Before(stroke) => {
        let id = content.add_stroke(*stroke);
        *self = Self::After(id);
      }
      _ => unreachable!(),
    };
  }

  fn rollback(&mut self, mut content: ContentAccessMut) {
    match std::mem::replace(self, Self::Invalid) {
      Self::After(id) => {
        let stroke = content.remove_stroke(id);
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
  pub fn new(id: StrokeId) -> Box<Self> {
    Box::new(Self::Before(id))
  }
}
#[typetag::serde]
impl ProtocolCommand for RemoveStrokeCommand {
  fn execute(&mut self, mut content: ContentAccessMut) {
    match *self {
      Self::Before(id) => {
        let stroke = content.remove_stroke(id);
        *self = Self::After(Box::new(stroke));
      }
      _ => unreachable!(),
    }
  }

  fn rollback(&mut self, mut content: ContentAccessMut) {
    match std::mem::replace(self, Self::Invalid) {
      Self::After(stroke) => {
        let id = content.add_stroke(*stroke);
        *self = Self::Before(id);
      }
      _ => unreachable!(),
    }
  }
}
