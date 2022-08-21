use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};

use super::content::PersistentContent;

use std::{
  collections::{HashMap, VecDeque},
  f32::consts::TAU,
};

#[typetag::serde(tag = "type")]
pub trait Command: DynClone {
  fn execute(&mut self, content: &mut PersistentContent) -> Result<(), ()>;
  fn rollback(&mut self, content: &mut PersistentContent);
}
dyn_clone::clone_trait_object!(Command);

#[derive(Default)]
pub struct ProtocolManager {
  protocol: ContentProtocol,
  queue: VecDeque<Todo>,
}

impl ProtocolManager {
  pub fn do_it(&mut self, cmd: Box<dyn Command>) {
    self.queue.push_back(Todo::Do(cmd))
  }

  pub fn undo(&mut self) {
    self.queue.push_back(Todo::Undo)
  }

  pub fn redo(&mut self) {
    self.queue.push_back(Todo::Redo)
  }

  pub fn switch_branch(&mut self, i: usize) {
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

  pub fn update(&mut self, content: &mut PersistentContent) {
    for todo in self.queue.drain(..) {
      match todo {
        Todo::Do(mut cmd) => {
          if let Ok(()) = cmd.execute(content) {
            let (new, new_id) = ProtocolNode::new(cmd, self.protocol.head);
            self.protocol.nodes.insert(new_id, new);
            self.protocol.head_node_mut().children.push(new_id);
            self.protocol.head = new_id;
          }
        }
        Todo::Undo => {
          self.protocol.head_node_mut().command.rollback(content);
          self.protocol.head = self.protocol.head_node().parent;
        }
        Todo::Redo => {
          let new_head = self.protocol.head_node_mut().children.last().copied();
          if let Some(new_head) = new_head {
            self
              .protocol
              .node_mut(new_head)
              .command
              .execute(content)
              .unwrap();
            self.protocol.head = new_head;
          }
        }
      }
    }
  }

  pub fn protocol(&self) -> &ContentProtocol {
    &self.protocol
  }

  pub fn protocol_mut(&mut self) -> &mut ContentProtocol {
    &mut self.protocol
  }
}

enum Todo {
  Do(Box<dyn Command>),
  Undo,
  Redo,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct NodeId(pub uuid::Uuid);
impl Default for NodeId {
  fn default() -> Self {
    let uuid = uuid::Uuid::new_v4();
    Self(uuid)
  }
}

/// Basically an undo tree
#[derive(Clone, Serialize, Deserialize)]
pub struct ContentProtocol {
  nodes: HashMap<NodeId, ProtocolNode>,
  head: NodeId,
}
impl Default for ContentProtocol {
  fn default() -> Self {
    let (root, root_id) = ProtocolNode::root();
    let mut nodes = HashMap::default();
    nodes.insert(root_id, root);
    let head = root_id;
    Self { nodes, head }
  }
}
impl ContentProtocol {
  #[allow(dead_code)]
  fn node(&self, id: NodeId) -> &ProtocolNode {
    self.nodes.get(&id).unwrap()
  }
  fn node_mut(&mut self, id: NodeId) -> &mut ProtocolNode {
    self.nodes.get_mut(&id).unwrap()
  }
  fn head_node(&self) -> &ProtocolNode {
    self.nodes.get(&self.head).unwrap()
  }
  fn head_node_mut(&mut self) -> &mut ProtocolNode {
    self.nodes.get_mut(&self.head).unwrap()
  }
}

#[derive(Clone, Serialize, Deserialize)]
struct ProtocolNode {
  command: Box<dyn Command>,
  creation_time: chrono::DateTime<chrono::Local>,

  parent: NodeId,
  children: Vec<NodeId>,
}
impl ProtocolNode {
  pub fn root() -> (Self, NodeId) {
    let id = NodeId::default();
    let command = Box::new(SentinelCommand);
    let creation_time = chrono::Local::now();
    let children = Vec::default();
    (
      Self {
        command,
        creation_time,
        parent: id,
        children,
      },
      id,
    )
  }
  pub fn new(command: Box<dyn Command>, parent: NodeId) -> (Self, NodeId) {
    let id = NodeId::default();
    let creation_time = chrono::Local::now();
    let children = Vec::default();
    (
      Self {
        command,
        creation_time,
        parent,
        children,
      },
      id,
    )
  }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct SentinelCommand;
#[typetag::serde]
impl Command for SentinelCommand {
  fn execute(&mut self, _content: &mut PersistentContent) -> Result<(), ()> {
    Ok(())
  }
  fn rollback(&mut self, _content: &mut PersistentContent) {}
}

// TODO: do we need full access (because where in this module)
// to the tree, to visualize it?
// Can we avoid this?
#[derive(Default)]
pub struct UndoTreeVisualizer {}
impl UndoTreeVisualizer {
  pub fn ui(&mut self, ui: &mut egui::Ui, manager: &mut ProtocolManager) {
    let size = egui::Vec2::splat(300.0);
    let (response, painter) = ui.allocate_painter(size, egui::Sense::click());
    let rect = response.rect;

    let c = rect.center();
    let r = rect.width() / 2.0 - 1.0;
    let rr = r / 10.0;
    let color = egui::Color32::from_gray(128);
    let stroke = egui::Stroke::new(4.0, color);

    let text = manager
      .protocol
      .head_node()
      .creation_time
      .format("%Y-%m-%d\n%H:%M:%S")
      .to_string();
    painter.circle_filled(c, rr, egui::Color32::GREEN);
    painter.text(
      c + egui::vec2(2.0 * rr, 0.0),
      egui::Align2::LEFT_CENTER,
      text,
      egui::FontId::default(),
      egui::Color32::WHITE,
    );

    let has_parent = manager.protocol.head_node().parent != manager.protocol.head;
    if has_parent {
      painter.line_segment(
        [c - egui::vec2(0.0, rr), c - egui::vec2(0.0, 6.0 * rr)],
        stroke,
      );
      let mut circle = egui::epaint::CircleShape::filled(
        c - egui::vec2(0.0, 6.0 * rr + rr),
        rr,
        egui::Color32::RED,
      );
      if let Some(cursor) = response.hover_pos() {
        if circle.visual_bounding_rect().contains(cursor) {
          circle.fill = egui::Color32::BLUE;
          if response.clicked() {
            manager.undo();
          }
        }
      }
      painter.add(circle);
    }
    let nchildren = manager.protocol.head_node().children.len();
    for i in 0..nchildren {
      let angle = if nchildren == 1 {
        3.0 / 4.0 * TAU
      } else {
        let fmax = (nchildren - 1) as f32;
        let fi = i as f32 / fmax;
        3.0 / 4.0 * TAU + (fi - 0.5) * TAU / 8.0
      };
      let angled = egui::Vec2::angled(angle);
      let line =
        egui::epaint::Shape::line_segment([c - rr * angled, c - 6.0 * rr * angled], stroke);
      painter.add(line);

      let mut circle =
        egui::epaint::CircleShape::filled(c - (6.0 + 1.0) * rr * angled, rr, egui::Color32::RED);
      if let Some(cursor) = response.hover_pos() {
        if circle.visual_bounding_rect().contains(cursor) {
          circle.fill = egui::Color32::BLUE;
          if response.clicked() {
            manager.switch_branch(i);
            manager.redo();
          }
        }
      }
      painter.add(circle);
    }
  }
}
