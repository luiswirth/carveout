use std::mem;

use crate::stroke::Stroke;

use super::{access::ContentAccessMut, StrokeId};

use dyn_clone::{clone_trait_object, DynClone};
use serde::{Deserialize, Serialize};

#[typetag::serde(tag = "type", content = "value")]
pub trait ProtocolCommand: DynClone + Send + Sync {
  fn execute(&mut self, content: ContentAccessMut);
  fn rollback(&mut self, content: ContentAccessMut);
}
clone_trait_object!(ProtocolCommand);

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
    match mem::replace(self, Self::Invalid) {
      Self::Before(stroke) => {
        let id = content.add_stroke(*stroke);
        *self = Self::After(id);
      }
      _ => unreachable!(),
    };
  }

  fn rollback(&mut self, mut content: ContentAccessMut) {
    match mem::replace(self, Self::Invalid) {
      Self::After(id) => {
        let stroke = content.remove_stroke(id);
        *self = Self::Before(Box::new(stroke));
      }
      _ => unreachable!(),
    };
  }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum RemoveStrokesCommand {
  Invalid,
  Before(Vec<StrokeId>),
  After(Vec<Stroke>),
}
impl RemoveStrokesCommand {
  pub fn single(id: StrokeId) -> Box<dyn ProtocolCommand> {
    Box::new(Self::Before(vec![id]))
  }
  pub fn multiple(ids: Vec<StrokeId>) -> Box<dyn ProtocolCommand> {
    Box::new(Self::Before(ids))
  }
}
#[typetag::serde]
impl ProtocolCommand for RemoveStrokesCommand {
  fn execute(&mut self, mut content: ContentAccessMut) {
    match mem::replace(self, Self::Invalid) {
      Self::Before(ids) => {
        let strokes = ids
          .into_iter()
          .map(|id| content.remove_stroke(id))
          .collect();
        *self = Self::After(strokes);
      }
      _ => unreachable!(),
    }
  }

  fn rollback(&mut self, mut content: ContentAccessMut) {
    match mem::replace(self, Self::Invalid) {
      Self::After(strokes) => {
        let ids = strokes.into_iter().map(|s| content.add_stroke(s)).collect();
        *self = Self::Before(ids);
      }
      _ => unreachable!(),
    }
  }
}
