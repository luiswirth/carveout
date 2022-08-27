//! Arena adapted from rapier.
use crate::util;

use serde::{Deserialize, Serialize};
use std::{
  cmp,
  iter::{self, Extend, FromIterator, FusedIterator},
  mem, ops, slice, vec,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Arena<T> {
  items: Vec<Entry<T>>,
  generation: u32,
  free_list_head: Option<u32>,
  len: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Entry<T> {
  Free { next_free: Option<u32> },
  Occupied { generation: u32, value: T },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ArenaIndex {
  index: u32,
  generation: u32,
}

impl Default for ArenaIndex {
  fn default() -> Self {
    Self::from_raw_parts(util::INVALID_U32, util::INVALID_U32)
  }
}

impl parry2d::partitioning::IndexedData for ArenaIndex {
  fn default() -> Self {
    Default::default()
  }

  fn index(&self) -> usize {
    self.into_raw_parts().0 as usize
  }
}

impl ArenaIndex {
  pub fn from_raw_parts(index: u32, generation: u32) -> ArenaIndex {
    ArenaIndex { index, generation }
  }

  pub fn into_raw_parts(self) -> (u32, u32) {
    (self.index, self.generation)
  }

  pub fn index(&self) -> u32 {
    self.index
  }
}

const DEFAULT_CAPACITY: usize = 4;

impl<T> Default for Arena<T> {
  fn default() -> Arena<T> {
    Arena::new()
  }
}

impl<T> Arena<T> {
  pub fn new() -> Arena<T> {
    Arena::with_capacity(DEFAULT_CAPACITY)
  }

  pub fn with_capacity(n: usize) -> Arena<T> {
    let n = cmp::max(n, 1);
    let mut arena = Arena {
      items: Vec::new(),
      generation: 0,
      free_list_head: None,
      len: 0,
    };
    arena.reserve(n);
    arena
  }

  pub fn clear(&mut self) {
    self.items.clear();

    let end = self.items.capacity() as u32;
    self.items.extend((0..end).map(|i| {
      if i == end - 1 {
        Entry::Free { next_free: None }
      } else {
        Entry::Free {
          next_free: Some(i + 1),
        }
      }
    }));
    self.free_list_head = Some(0);
    self.len = 0;
  }

  #[inline]
  pub fn try_insert(&mut self, value: T) -> Result<ArenaIndex, T> {
    match self.try_alloc_next_index() {
      None => Err(value),
      Some(index) => {
        self.items[index.index as usize] = Entry::Occupied {
          generation: self.generation,
          value,
        };
        Ok(index)
      }
    }
  }

  #[inline]
  pub fn try_insert_with<F: FnOnce(ArenaIndex) -> T>(
    &mut self,
    create: F,
  ) -> Result<ArenaIndex, F> {
    match self.try_alloc_next_index() {
      None => Err(create),
      Some(index) => {
        self.items[index.index as usize] = Entry::Occupied {
          generation: self.generation,
          value: create(index),
        };
        Ok(index)
      }
    }
  }

  #[inline]
  fn try_alloc_next_index(&mut self) -> Option<ArenaIndex> {
    match self.free_list_head {
      None => None,
      Some(i) => match self.items[i as usize] {
        Entry::Occupied { .. } => panic!("corrupt free list"),
        Entry::Free { next_free } => {
          self.free_list_head = next_free;
          self.len += 1;
          Some(ArenaIndex {
            index: i as u32,
            generation: self.generation,
          })
        }
      },
    }
  }

  #[inline]
  pub fn insert(&mut self, value: T) -> ArenaIndex {
    match self.try_insert(value) {
      Ok(i) => i,
      Err(value) => self.insert_slow_path(value),
    }
  }

  #[inline]
  pub fn insert_with(&mut self, create: impl FnOnce(ArenaIndex) -> T) -> ArenaIndex {
    match self.try_insert_with(create) {
      Ok(i) => i,
      Err(create) => self.insert_with_slow_path(create),
    }
  }

  #[inline(never)]
  fn insert_slow_path(&mut self, value: T) -> ArenaIndex {
    let len = self.items.len();
    self.reserve(len);
    self
      .try_insert(value)
      .map_err(|_| ())
      .expect("inserting will always succeed after reserving additional space")
  }

  #[inline(never)]
  fn insert_with_slow_path(&mut self, create: impl FnOnce(ArenaIndex) -> T) -> ArenaIndex {
    let len = self.items.len();
    self.reserve(len);
    self
      .try_insert_with(create)
      .map_err(|_| ())
      .expect("inserting will always succeed after reserving additional space")
  }

  pub fn remove(&mut self, i: ArenaIndex) -> Option<T> {
    if i.index >= self.items.len() as u32 {
      return None;
    }

    match self.items[i.index as usize] {
      Entry::Occupied { generation, .. } if i.generation == generation => {
        let entry = mem::replace(
          &mut self.items[i.index as usize],
          Entry::Free {
            next_free: self.free_list_head,
          },
        );
        self.generation += 1;
        self.free_list_head = Some(i.index);
        self.len -= 1;

        match entry {
          Entry::Occupied {
            generation: _,
            value,
          } => Some(value),
          _ => unreachable!(),
        }
      }
      _ => None,
    }
  }

  pub fn retain(&mut self, mut predicate: impl FnMut(ArenaIndex, &mut T) -> bool) {
    for i in 0..self.capacity() as u32 {
      let remove = match &mut self.items[i as usize] {
        Entry::Occupied { generation, value } => {
          let index = ArenaIndex {
            index: i,
            generation: *generation,
          };
          if predicate(index, value) {
            None
          } else {
            Some(index)
          }
        }

        _ => None,
      };
      if let Some(index) = remove {
        self.remove(index);
      }
    }
  }

  pub fn contains(&self, i: ArenaIndex) -> bool {
    self.get(i).is_some()
  }

  pub fn get(&self, i: ArenaIndex) -> Option<&T> {
    match self.items.get(i.index as usize) {
      Some(Entry::Occupied { generation, value }) if *generation == i.generation => Some(value),
      _ => None,
    }
  }

  pub fn get_mut(&mut self, i: ArenaIndex) -> Option<&mut T> {
    match self.items.get_mut(i.index as usize) {
      Some(Entry::Occupied { generation, value }) if *generation == i.generation => Some(value),
      _ => None,
    }
  }

  pub fn get2_mut(&mut self, i1: ArenaIndex, i2: ArenaIndex) -> (Option<&mut T>, Option<&mut T>) {
    let len = self.items.len() as u32;

    if i1.index == i2.index {
      assert!(i1.generation != i2.generation);

      if i1.generation > i2.generation {
        return (self.get_mut(i1), None);
      }
      return (None, self.get_mut(i2));
    }

    if i1.index >= len {
      return (None, self.get_mut(i2));
    } else if i2.index >= len {
      return (self.get_mut(i1), None);
    }

    let (raw_item1, raw_item2) = {
      let (xs, ys) = self
        .items
        .split_at_mut(cmp::max(i1.index, i2.index) as usize);
      if i1.index < i2.index {
        (&mut xs[i1.index as usize], &mut ys[0])
      } else {
        (&mut ys[0], &mut xs[i2.index as usize])
      }
    };

    let item1 = match raw_item1 {
      Entry::Occupied { generation, value } if *generation == i1.generation => Some(value),
      _ => None,
    };

    let item2 = match raw_item2 {
      Entry::Occupied { generation, value } if *generation == i2.generation => Some(value),
      _ => None,
    };

    (item1, item2)
  }

  pub fn len(&self) -> usize {
    self.len
  }

  pub fn is_empty(&self) -> bool {
    self.len == 0
  }

  pub fn capacity(&self) -> usize {
    self.items.len()
  }

  pub fn reserve(&mut self, additional_capacity: usize) {
    let start = self.items.len();
    let end = self.items.len() + additional_capacity;
    let old_head = self.free_list_head;
    self.items.reserve_exact(additional_capacity);
    self.items.extend((start..end).map(|i| {
      if i == end - 1 {
        Entry::Free {
          next_free: old_head,
        }
      } else {
        Entry::Free {
          next_free: Some(i as u32 + 1),
        }
      }
    }));
    self.free_list_head = Some(start as u32);
  }

  pub fn iter(&self) -> Iter<T> {
    Iter {
      len: self.len,
      inner: self.items.iter().enumerate(),
    }
  }

  pub fn iter_mut(&mut self) -> IterMut<T> {
    IterMut {
      len: self.len,
      inner: self.items.iter_mut().enumerate(),
    }
  }

  pub fn drain(&mut self) -> Drain<T> {
    Drain {
      inner: self.items.drain(..).enumerate(),
    }
  }

  pub fn get_unknown_gen(&self, i: u32) -> Option<(&T, ArenaIndex)> {
    match self.items.get(i as usize) {
      Some(Entry::Occupied { generation, value }) => Some((
        value,
        ArenaIndex {
          generation: *generation,
          index: i,
        },
      )),
      _ => None,
    }
  }

  pub fn get_unknown_gen_mut(&mut self, i: u32) -> Option<(&mut T, ArenaIndex)> {
    match self.items.get_mut(i as usize) {
      Some(Entry::Occupied { generation, value }) => Some((
        value,
        ArenaIndex {
          generation: *generation,
          index: i,
        },
      )),
      _ => None,
    }
  }
}

impl<T> IntoIterator for Arena<T> {
  type Item = T;
  type IntoIter = IntoIter<T>;
  fn into_iter(self) -> Self::IntoIter {
    IntoIter {
      len: self.len,
      inner: self.items.into_iter(),
    }
  }
}

#[derive(Clone, Debug)]
pub struct IntoIter<T> {
  len: usize,
  inner: vec::IntoIter<Entry<T>>,
}

impl<T> Iterator for IntoIter<T> {
  type Item = T;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      match self.inner.next() {
        Some(Entry::Free { .. }) => continue,
        Some(Entry::Occupied { value, .. }) => {
          self.len -= 1;
          return Some(value);
        }
        None => {
          debug_assert_eq!(self.len, 0);
          return None;
        }
      }
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    (self.len, Some(self.len))
  }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    loop {
      match self.inner.next_back() {
        Some(Entry::Free { .. }) => continue,
        Some(Entry::Occupied { value, .. }) => {
          self.len -= 1;
          return Some(value);
        }
        None => {
          debug_assert_eq!(self.len, 0);
          return None;
        }
      }
    }
  }
}

impl<T> ExactSizeIterator for IntoIter<T> {
  fn len(&self) -> usize {
    self.len
  }
}

impl<T> FusedIterator for IntoIter<T> {}

impl<'a, T> IntoIterator for &'a Arena<T> {
  type Item = (ArenaIndex, &'a T);
  type IntoIter = Iter<'a, T>;
  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

#[derive(Clone, Debug)]
pub struct Iter<'a, T: 'a> {
  len: usize,
  inner: iter::Enumerate<slice::Iter<'a, Entry<T>>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
  type Item = (ArenaIndex, &'a T);

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      match self.inner.next() {
        Some((_, &Entry::Free { .. })) => continue,
        Some((
          index,
          &Entry::Occupied {
            generation,
            ref value,
          },
        )) => {
          self.len -= 1;
          let idx = ArenaIndex {
            index: index as u32,
            generation,
          };
          return Some((idx, value));
        }
        None => {
          debug_assert_eq!(self.len, 0);
          return None;
        }
      }
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    (self.len, Some(self.len))
  }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    loop {
      match self.inner.next_back() {
        Some((_, &Entry::Free { .. })) => continue,
        Some((
          index,
          &Entry::Occupied {
            generation,
            ref value,
          },
        )) => {
          self.len -= 1;
          let idx = ArenaIndex {
            index: index as u32,
            generation,
          };
          return Some((idx, value));
        }
        None => {
          debug_assert_eq!(self.len, 0);
          return None;
        }
      }
    }
  }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
  fn len(&self) -> usize {
    self.len
  }
}

impl<'a, T> FusedIterator for Iter<'a, T> {}

impl<'a, T> IntoIterator for &'a mut Arena<T> {
  type Item = (ArenaIndex, &'a mut T);
  type IntoIter = IterMut<'a, T>;
  fn into_iter(self) -> Self::IntoIter {
    self.iter_mut()
  }
}

#[derive(Debug)]
pub struct IterMut<'a, T: 'a> {
  len: usize,
  inner: iter::Enumerate<slice::IterMut<'a, Entry<T>>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
  type Item = (ArenaIndex, &'a mut T);

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      match self.inner.next() {
        Some((_, &mut Entry::Free { .. })) => continue,
        Some((
          index,
          &mut Entry::Occupied {
            generation,
            ref mut value,
          },
        )) => {
          self.len -= 1;
          let idx = ArenaIndex {
            index: index as u32,
            generation,
          };
          return Some((idx, value));
        }
        None => {
          debug_assert_eq!(self.len, 0);
          return None;
        }
      }
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    (self.len, Some(self.len))
  }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    loop {
      match self.inner.next_back() {
        Some((_, &mut Entry::Free { .. })) => continue,
        Some((
          index,
          &mut Entry::Occupied {
            generation,
            ref mut value,
          },
        )) => {
          self.len -= 1;
          let idx = ArenaIndex {
            index: index as u32,
            generation,
          };
          return Some((idx, value));
        }
        None => {
          debug_assert_eq!(self.len, 0);
          return None;
        }
      }
    }
  }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
  fn len(&self) -> usize {
    self.len
  }
}

impl<'a, T> FusedIterator for IterMut<'a, T> {}

#[derive(Debug)]
pub struct Drain<'a, T: 'a> {
  inner: iter::Enumerate<vec::Drain<'a, Entry<T>>>,
}

impl<'a, T> Iterator for Drain<'a, T> {
  type Item = (ArenaIndex, T);

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      match self.inner.next() {
        Some((_, Entry::Free { .. })) => continue,
        Some((index, Entry::Occupied { generation, value })) => {
          let idx = ArenaIndex {
            index: index as u32,
            generation,
          };
          return Some((idx, value));
        }
        None => return None,
      }
    }
  }
}

impl<T> Extend<T> for Arena<T> {
  fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
    for t in iter {
      self.insert(t);
    }
  }
}

impl<T> FromIterator<T> for Arena<T> {
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    let iter = iter.into_iter();
    let (lower, upper) = iter.size_hint();
    let cap = upper.unwrap_or(lower);
    let cap = cmp::max(cap, 1);
    let mut arena = Arena::with_capacity(cap);
    arena.extend(iter);
    arena
  }
}

impl<T> ops::Index<ArenaIndex> for Arena<T> {
  type Output = T;

  fn index(&self, index: ArenaIndex) -> &Self::Output {
    self.get(index).expect("No element at index")
  }
}

impl<T> ops::IndexMut<ArenaIndex> for Arena<T> {
  fn index_mut(&mut self, index: ArenaIndex) -> &mut Self::Output {
    self.get_mut(index).expect("No element at index")
  }
}
