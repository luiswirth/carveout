pub mod command;
pub mod protocol;

use self::{
  command::ProtocolCommand,
  protocol::{Protocol, ProtocolNode},
};

use super::stroke::{Stroke, StrokeId};

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

#[derive(Default)]
pub struct ContentManager {
  persistent: PersistentContent,
  protocol: Protocol,
  pending_changes: VecDeque<PendingChange>,

  ongoing: OngoingContent,
}

impl ContentManager {
  pub fn schedule_cmd(&mut self, cmd: Box<dyn ProtocolCommand>) {
    self.pending_changes.push_back(PendingChange::Do(cmd))
  }

  pub fn schedule_undo(&mut self) {
    self.pending_changes.push_back(PendingChange::Undo)
  }

  pub fn schedule_redo(&mut self) {
    self.pending_changes.push_back(PendingChange::Redo)
  }

  pub fn switch_protocol_branch(&mut self, i: usize) {
    let head = self.protocol.head_node_mut();
    let i_last = head.children.len() - 1;
    head.children.swap(i, i_last);
  }

  pub fn undoable(&self) -> bool {
    self.protocol.head_node().parent != self.protocol.head
  }

  pub fn redoable(&self) -> bool {
    !self.protocol.head_node().children.is_empty()
  }

  pub fn update(&mut self) {
    for todo in self.pending_changes.drain(..) {
      match todo {
        PendingChange::Do(mut cmd) => {
          if let Ok(()) = cmd.execute(&mut self.persistent) {
            let (new, new_id) = ProtocolNode::new(cmd, self.protocol.head);
            self.protocol.nodes.insert(new_id, new);
            self.protocol.head_node_mut().children.push(new_id);
            self.protocol.head = new_id;
          }
        }
        PendingChange::Undo => {
          self
            .protocol
            .head_node_mut()
            .command
            .rollback(&mut self.persistent);
          self.protocol.head = self.protocol.head_node().parent;
        }
        PendingChange::Redo => {
          let new_head = self.protocol.head_node_mut().children.last().copied();
          if let Some(new_head) = new_head {
            self
              .protocol
              .node_mut(new_head)
              .command
              .execute(&mut self.persistent)
              .unwrap();
            self.protocol.head = new_head;
          }
        }
      }
    }
  }
}
impl ContentManager {
  pub fn persistent(&self) -> &PersistentContent {
    &self.persistent
  }

  pub fn persistent_mut(&mut self) -> &mut PersistentContent {
    &mut self.persistent
  }

  pub fn ongoing(&mut self) -> &mut OngoingContent {
    &mut self.ongoing
  }

  pub fn persistent_ongoing_mut(&mut self) -> (&mut PersistentContent, &mut OngoingContent) {
    (&mut self.persistent, &mut self.ongoing)
  }

  pub fn protocol(&self) -> &Protocol {
    &self.protocol
  }

  pub fn protocol_mut(&mut self) -> &mut Protocol {
    &mut self.protocol
  }
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

/// Can be freely mutated.
#[derive(Default)]
pub struct OngoingContent {
  pub stroke: Option<Stroke>,
}

enum PendingChange {
  Do(Box<dyn ProtocolCommand>),
  Undo,
  Redo,
}