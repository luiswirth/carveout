use std::mem;

use crate::stroke::Stroke;

use super::{access::ContentAccessMut, StrokeId};

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum ProtocolCommand {
  Sentinel,
  AddStrokeCommand(AddStrokeCommand),
  RemoveStrokesCommand(RemoveStrokesCommand),
}
impl ProtocolCommand {
  pub fn execute(&mut self, content: ContentAccessMut) {
    match self {
      ProtocolCommand::Sentinel => {}
      ProtocolCommand::AddStrokeCommand(cmd) => cmd.execute(content),
      ProtocolCommand::RemoveStrokesCommand(cmd) => cmd.execute(content),
    }
  }

  pub fn rollback(&mut self, content: ContentAccessMut) {
    match self {
      ProtocolCommand::Sentinel => {}
      ProtocolCommand::AddStrokeCommand(cmd) => cmd.rollback(content),
      ProtocolCommand::RemoveStrokesCommand(cmd) => cmd.rollback(content),
    }
  }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AddStrokeCommand {
  Invalid,
  Before(Box<Stroke>),
  After(StrokeId),
}

impl AddStrokeCommand {
  pub fn new(stroke: Stroke) -> ProtocolCommand {
    ProtocolCommand::AddStrokeCommand(Self::Before(Box::new(stroke)))
  }
}
impl AddStrokeCommand {
  pub fn execute(&mut self, mut content: ContentAccessMut) {
    match mem::replace(self, Self::Invalid) {
      Self::Before(stroke) => {
        let id = content.add_stroke(*stroke);
        *self = Self::After(id);
      }
      _ => unreachable!(),
    };
  }

  pub fn rollback(&mut self, mut content: ContentAccessMut) {
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
  pub fn single(id: StrokeId) -> ProtocolCommand {
    ProtocolCommand::RemoveStrokesCommand(Self::Before(vec![id]))
  }
  pub fn multiple(ids: Vec<StrokeId>) -> ProtocolCommand {
    ProtocolCommand::RemoveStrokesCommand(Self::Before(ids))
  }

  pub fn execute(&mut self, mut content: ContentAccessMut) {
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
  pub fn rollback(&mut self, mut content: ContentAccessMut) {
    match mem::replace(self, Self::Invalid) {
      Self::After(strokes) => {
        let ids = strokes.into_iter().map(|s| content.add_stroke(s)).collect();
        *self = Self::Before(ids);
      }
      _ => unreachable!(),
    }
  }
}
