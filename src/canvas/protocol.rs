use super::content::PersistentContent;

use std::{
  cell::RefCell,
  collections::VecDeque,
  f32::consts::TAU,
  rc::{Rc, Weak},
};

#[typetag::serde(tag = "type")]
pub trait Command {
  fn execute(&mut self, content: &mut PersistentContent);
  fn rollback(&mut self, content: &mut PersistentContent);
}

enum Todo {
  Do(Box<dyn Command>),
  Undo,
  Redo,
}

/// Basically an undo tree
#[derive(Clone)]
pub struct ContentProtocol {
  root: StrongLink,
  head: WeakLink,
}
impl Default for ContentProtocol {
  fn default() -> Self {
    let root = TreeNode::new_root_link();
    let head = Rc::downgrade(&root);
    Self { root, head }
  }
}

#[derive(Default)]
pub struct ProtocolManager {
  queue: VecDeque<Todo>,
  protocol: ContentProtocol,
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
    let head = self.protocol.head.upgrade().unwrap();
    let mut head = head.borrow_mut();
    let i_last = head.children.len() - 1;
    head.children.swap(i, i_last);
  }

  pub fn undoable(&self) -> bool {
    let head = self.protocol.head.upgrade().unwrap();
    !Rc::ptr_eq(&head, &self.protocol.root)
  }

  pub fn redoable(&self) -> bool {
    let head = self.protocol.head.upgrade().unwrap();
    let head = head.borrow();
    !head.children.is_empty()
  }

  pub fn update(&mut self, content: &mut PersistentContent) {
    for todo in self.queue.drain(..) {
      match todo {
        Todo::Do(mut cmd) => {
          cmd.execute(content);
          let new_strong = TreeNode::new_link(cmd, self.protocol.head.clone());
          let new_weak = Rc::downgrade(&new_strong);
          let head = self.protocol.head.upgrade().unwrap();
          let mut head = head.borrow_mut();
          head.children.push(new_strong);
          self.protocol.head = new_weak;
        }
        Todo::Undo => {
          let head = self.protocol.head.upgrade().unwrap();
          let mut head = head.borrow_mut();
          head.command.rollback(content);
          self.protocol.head = head.parent.clone();
        }
        Todo::Redo => {
          let head = self.protocol.head.upgrade().unwrap();
          let head = head.borrow_mut();
          if let Some(new_head) = head.children.last() {
            new_head.borrow_mut().command.execute(content);
            self.protocol.head = Rc::downgrade(new_head);
          }
        }
      }
    }
  }

  #[allow(dead_code)]
  pub fn protocol(&self) -> &ContentProtocol {
    &self.protocol
  }

  #[allow(dead_code)]
  pub fn protocol_mut(&mut self) -> &mut ContentProtocol {
    &mut self.protocol
  }
}

type StrongLink = Rc<RefCell<TreeNode>>;
type WeakLink = Weak<RefCell<TreeNode>>;

struct TreeNode {
  command: Box<dyn Command>,
  creation_time: chrono::DateTime<chrono::Local>,

  parent: WeakLink,
  children: Vec<StrongLink>,
}
impl std::fmt::Debug for TreeNode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("TreeNode")
      .field("parent", &self.parent)
      .field("children", &self.children)
      .finish()
  }
}

impl TreeNode {
  fn new_link(command: Box<dyn Command>, parent: WeakLink) -> StrongLink {
    let creation_time = chrono::Local::now();
    let children = Vec::new();
    Rc::new(RefCell::new(Self {
      creation_time,
      command,
      parent,
      children,
    }))
  }
  fn new_root_link() -> StrongLink {
    let command = Box::new(SentinelCommand);
    let creation_time = chrono::Local::now();
    let children = Vec::new();
    Rc::new_cyclic(|itself| {
      RefCell::new(Self {
        creation_time,
        command,
        parent: itself.to_owned(),
        children,
      })
    })
  }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SentinelCommand;
#[typetag::serde]
impl Command for SentinelCommand {
  fn execute(&mut self, _content: &mut PersistentContent) {}
  fn rollback(&mut self, _content: &mut PersistentContent) {}
}

// TODO: do we need full access (because where in this module)
// to the tree, to visualize it?
// Can we avoid this?
#[derive(Default)]
pub struct UndoTreeVisualizer {}
impl UndoTreeVisualizer {
  pub fn ui(&mut self, ui: &mut egui::Ui, protocol_manager: &mut ProtocolManager) {
    let size = egui::Vec2::splat(300.0);
    let (response, painter) = ui.allocate_painter(size, egui::Sense::click());
    let rect = response.rect;

    let c = rect.center();
    let r = rect.width() / 2.0 - 1.0;
    let rr = r / 10.0;
    let color = egui::Color32::from_gray(128);
    let stroke = egui::Stroke::new(4.0, color);

    let head = protocol_manager.protocol.head.upgrade().unwrap();
    let text = head
      .borrow()
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

    let has_parent = !Rc::ptr_eq(&head, &protocol_manager.protocol.root);
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
            protocol_manager.undo();
          }
        }
      }
      painter.add(circle);
    }
    let nchildren = head.borrow().children.len();
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
            protocol_manager.switch_branch(i);
            protocol_manager.redo();
          }
        }
      }
      painter.add(circle);
    }
  }
}
