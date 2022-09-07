pub mod access;
pub mod arena;
pub mod command;
pub mod protocol;

use self::{
  access::{ContentAccess, ContentAccessMut, ContentDelta},
  arena::{Arena, ArenaIndex},
  command::ProtocolCommand,
  protocol::{Protocol, ProtocolNode, ProtocolNodeId},
};
use super::stroke::Stroke;

use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct ContentManager {
  content: Content,
  protocol: Protocol,
  delta: ContentDelta,
}

impl ContentManager {
  pub fn run_cmd(&mut self, mut cmd: ProtocolCommand) {
    cmd.execute(self.access_mut());
    let new = ProtocolNode::new(cmd, self.protocol.head);
    let new_id = ProtocolNodeId(u32::try_from(self.protocol.nodes.len()).unwrap());
    self.protocol.nodes.push(new);
    let old_head = self.protocol.head_node_mut();
    old_head.children.push(new_id);
    old_head.selected_child = Some(old_head.children.len() - 1);
    self.protocol.head = new_id;
  }

  /// If there is nothing to undo then it does nothing.
  pub fn undo_cmd(&mut self) {
    let access_mut = ContentAccessMut {
      content: &mut self.content,
      delta: &mut self.delta,
    };
    self.protocol.head_node_mut().command.rollback(access_mut);
    self.protocol.head = self.protocol.head_node().parent;
  }

  /// If there is nothing to redo then it does nothing.
  pub fn redo_cmd(&mut self) {
    let head = self.protocol.head_node_mut();
    let access_mut = ContentAccessMut {
      content: &mut self.content,
      delta: &mut self.delta,
    };
    if let Some(selected_child) = head.selected_child {
      let selected_child = head.children[selected_child];
      self
        .protocol
        .node_mut(selected_child)
        .command
        .execute(access_mut);
      self.protocol.head = selected_child;
    }
  }

  pub fn switch_protocol_branch(&mut self, child_index: usize) {
    let head = self.protocol.head_node_mut();
    assert!(child_index < head.children.len());
    head.selected_child = Some(child_index);
  }

  pub fn undoable(&self) -> bool {
    self.protocol.head_node().parent != self.protocol.head
  }

  pub fn redoable(&self) -> bool {
    !self.protocol.head_node().children.is_empty()
  }
}
impl ContentManager {
  pub fn access(&self) -> ContentAccess {
    ContentAccess {
      content: &self.content,
    }
  }

  pub fn access_mut(&mut self) -> ContentAccessMut {
    ContentAccessMut {
      content: &mut self.content,
      delta: &mut self.delta,
    }
  }

  pub fn delta(&self) -> &ContentDelta {
    &self.delta
  }

  pub fn reset_delta(&mut self) {
    self.delta.clear();
  }

  pub fn replace(&mut self, content: Content, protocol: Protocol) {
    self.delta.strokes.removed = self
      .content
      .strokes
      .iter()
      .map(|(id, _)| StrokeId(id))
      .collect();
    self.delta.strokes.added = content.strokes.iter().map(|(id, _)| StrokeId(id)).collect();

    self.content = content;
    self.protocol = protocol;
  }

  pub fn clone(&self) -> (Content, Protocol) {
    (self.content.clone(), self.protocol.clone())
  }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Content {
  strokes: Arena<Stroke>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StrokeId(pub ArenaIndex);
