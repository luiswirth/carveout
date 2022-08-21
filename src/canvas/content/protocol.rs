use std::{collections::HashMap, f32::consts::TAU};

use serde::{Deserialize, Serialize};

use super::{command::ProtocolCommand, ContentManager};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(super) struct ProtocolNodeId(pub uuid::Uuid);
impl Default for ProtocolNodeId {
  fn default() -> Self {
    let uuid = uuid::Uuid::new_v4();
    Self(uuid)
  }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Protocol {
  pub(super) nodes: HashMap<ProtocolNodeId, ProtocolNode>,
  pub(super) head: ProtocolNodeId,
}
impl Default for Protocol {
  fn default() -> Self {
    let (root, root_id) = ProtocolNode::root();
    let mut nodes = HashMap::default();
    nodes.insert(root_id, root);
    let head = root_id;
    Self { nodes, head }
  }
}
impl Protocol {
  pub(super) fn node_mut(&mut self, id: ProtocolNodeId) -> &mut ProtocolNode {
    self.nodes.get_mut(&id).unwrap()
  }
  pub(super) fn head_node(&self) -> &ProtocolNode {
    self.nodes.get(&self.head).unwrap()
  }
  pub(super) fn head_node_mut(&mut self) -> &mut ProtocolNode {
    self.nodes.get_mut(&self.head).unwrap()
  }
}

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct ProtocolNode {
  pub(super) command: Box<dyn ProtocolCommand>,
  pub(super) creation_time: chrono::DateTime<chrono::Local>,

  pub(super) parent: ProtocolNodeId,
  pub(super) children: Vec<ProtocolNodeId>,
}
impl ProtocolNode {
  pub fn root() -> (Self, ProtocolNodeId) {
    let id = ProtocolNodeId::default();
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
  pub fn new(command: Box<dyn ProtocolCommand>, parent: ProtocolNodeId) -> (Self, ProtocolNodeId) {
    let id = ProtocolNodeId::default();
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
impl ProtocolCommand for SentinelCommand {
  fn execute(&mut self, _content: &mut super::PersistentContent) -> Result<(), ()> {
    Ok(())
  }
  fn rollback(&mut self, _content: &mut super::PersistentContent) {}
}

#[derive(Default)]
pub struct ProtocolUi {}
impl ProtocolUi {
  pub fn ui(&mut self, ui: &mut egui::Ui, manager: &mut ContentManager) {
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
            manager.schedule_undo();
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
            manager.switch_protocol_branch(i);
            manager.schedule_redo();
          }
        }
      }
      painter.add(circle);
    }
  }
}
