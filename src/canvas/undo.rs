use std::{
  cell::RefCell,
  rc::{Rc, Weak},
};

use super::content::PersistentContent;
use std::f32::consts::TAU;

pub trait Command {
  fn execute(&mut self, content: &mut PersistentContent);
  fn rollback(&mut self, content: &mut PersistentContent);
}

pub struct UndoTree {
  root: StrongLink,
  head: WeakLink,
}

impl UndoTree {
  pub fn new() -> Self {
    let root = TreeNode::new_root_link();
    let head = Rc::downgrade(&root);
    Self { root, head }
  }

  // TODO: introduce command queue
  // delay executing commands until "update"
  // => don't require content to be passed in here.
  // TODO: never store identical siblings (e.g. undoing stroke delete and deleting the same stroke again)
  pub fn do_it(&mut self, mut command: Box<dyn Command>, content: &mut PersistentContent) {
    command.execute(content);
    let new_strong = TreeNode::new_link(command, self.head.clone());
    let new_weak = Rc::downgrade(&new_strong);
    let head = self.head.upgrade().unwrap();
    let mut head = head.borrow_mut();
    head.children.push(new_strong);
    self.head = new_weak;
  }

  /// does nothing if not undoable
  pub fn undo(&mut self, content: &mut PersistentContent) {
    let head = self.head.upgrade().unwrap();
    let mut head = head.borrow_mut();
    head.command.rollback(content);
    self.head = head.parent.clone();
  }

  /// does nothing if not redoable
  pub fn redo(&mut self, content: &mut PersistentContent) {
    let head = self.head.upgrade().unwrap();
    let head = head.borrow_mut();
    if let Some(new_head) = head.children.last() {
      new_head.borrow_mut().command.execute(content);
      self.head = Rc::downgrade(new_head);
    }
  }

  pub fn switch_branch(&mut self, i: usize) {
    let head = self.head.upgrade().unwrap();
    let mut head = head.borrow_mut();
    let i_last = head.children.len() - 1;
    head.children.swap(i, i_last);
  }

  pub fn undoable(&self) -> bool {
    let head = self.head.upgrade().unwrap();
    !Rc::ptr_eq(&head, &self.root)
  }

  pub fn redoable(&self) -> bool {
    let head = self.head.upgrade().unwrap();
    let head = head.borrow();
    !head.children.is_empty()
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

struct SentinelCommand;
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
  pub fn ui(
    &mut self,
    ui: &mut egui::Ui,
    content: &mut PersistentContent,
    undo_tree: &mut UndoTree,
  ) {
    let size = egui::Vec2::splat(300.0);
    let (response, painter) = ui.allocate_painter(size, egui::Sense::click());
    let rect = response.rect;

    let c = rect.center();
    let r = rect.width() / 2.0 - 1.0;
    let rr = r / 10.0;
    let color = egui::Color32::from_gray(128);
    let stroke = egui::Stroke::new(4.0, color);

    let head = undo_tree.head.upgrade().unwrap();
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

    let has_parent = !Rc::ptr_eq(&head, &undo_tree.root);
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
            undo_tree.undo(content);
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
            undo_tree.switch_branch(i);
            undo_tree.redo(content);
          }
        }
      }
      painter.add(circle);
    }
  }
}
