use super::{Content, StrokeId};

use crate::stroke::Stroke;

pub struct ContentAccess<'a> {
  pub(super) content: &'a Content,
}
impl<'a> ContentAccess<'a> {
  pub fn stroke(&self, id: StrokeId) -> &Stroke {
    self.content.strokes.get(id.0).unwrap()
  }

  pub fn strokes(&self) -> impl Iterator<Item = (StrokeId, &Stroke)> {
    self
      .content
      .strokes
      .iter()
      .map(|(id, stroke)| (StrokeId(id), stroke))
  }
}

pub struct ContentAccessMut<'a> {
  pub(super) content: &'a mut Content,
  pub(super) delta: &'a mut ContentDelta,
}

/// Methods for everybody
impl<'a> ContentAccessMut<'a> {
  pub fn modify_stroke(&mut self, id: StrokeId) -> &mut Stroke {
    self.delta.strokes.modified.push(id);
    self.content.strokes.get_mut(id.0).unwrap()
  }
}

/// Methods for content module
impl<'a> ContentAccessMut<'a> {
  pub(super) fn add_stroke(&mut self, stroke: Stroke) -> StrokeId {
    let id = self.content.strokes.insert(stroke);
    let id = StrokeId(id);
    self.delta.strokes.added.push(id);
    id
  }

  pub(super) fn remove_stroke(&mut self, id: StrokeId) -> Stroke {
    let result = self.content.strokes.remove(id.0);

    // If failed to remove id, try removing different generation
    match result {
      Some(stroke) => {
        self.delta.strokes.removed.push(id);
        stroke
      }
      None => {
        let alternative_id = self
          .content
          .strokes
          .get_unknown_gen(id.0.index())
          .unwrap()
          .1;
        let alternative_id = StrokeId(alternative_id);
        self.delta.strokes.removed.push(alternative_id);
        self.content.strokes.remove(alternative_id.0).unwrap()
      }
    }
  }
}

#[derive(Default)]
pub struct ContentDelta {
  pub strokes: StrokeDelta,
}
impl ContentDelta {
  pub fn clear(&mut self) {
    self.strokes.clear();
  }
}

#[derive(Default)]
pub struct StrokeDelta {
  pub added: Vec<StrokeId>,
  pub modified: Vec<StrokeId>,
  pub removed: Vec<StrokeId>,
}
impl StrokeDelta {
  pub fn clear(&mut self) {
    self.added.clear();
    self.modified.clear();
    self.removed.clear();
  }
}
