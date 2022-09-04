use crate::{camera::Camera, spaces::*};

use std::collections::{HashMap, HashSet};
use winit::{
  event::{
    ElementState, ModifiersState, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
  },
  window::Window,
};

#[derive(Default)]
pub struct InputState {
  pub prev: InputsSnapshot,
  pub curr: InputsSnapshot,

  pub cursor_pos_left_clicked: Option<PointInSpaces>,
  pub mouse_scroll_delta: Option<VectorInSpaces>,
  pub multi_touch_movement: Option<TouchMovement>,
}

#[derive(Default, Clone)]
pub struct InputsSnapshot {
  pub pressed: HashSet<VirtualKeyCode>,
  pub clicked: HashSet<MouseButton>,
  pub touches: HashMap<TouchId, Touch>,
  pub multi_touch: Option<MultiTouch>,
  pub modifiers: ModifiersState,
  pub cursor_pos: Option<PointInSpaces>,
}

#[allow(dead_code)]
impl InputState {
  pub fn key(&self, key: VirtualKeyCode) -> ElementState {
    match self.curr.pressed.contains(&key) {
      true => ElementState::Pressed,
      false => ElementState::Released,
    }
  }

  pub fn key_prev(&self, key: VirtualKeyCode) -> ElementState {
    match self.prev.pressed.contains(&key) {
      true => ElementState::Pressed,
      false => ElementState::Released,
    }
  }

  pub fn button(&self, button: MouseButton) -> ElementState {
    match self.curr.clicked.contains(&button) {
      true => ElementState::Pressed,
      false => ElementState::Released,
    }
  }

  pub fn button_prev(&self, button: MouseButton) -> ElementState {
    match self.prev.clicked.contains(&button) {
      true => ElementState::Pressed,
      false => ElementState::Released,
    }
  }
}

#[allow(dead_code)]
impl InputState {
  pub fn is_pressed(&self, key: VirtualKeyCode) -> bool {
    self.key(key) == ElementState::Pressed
  }

  pub fn was_pressed(&self, key: VirtualKeyCode) -> bool {
    self.key_prev(key) == ElementState::Pressed
  }

  pub fn is_unpressed(&self, key: VirtualKeyCode) -> bool {
    self.key(key) == ElementState::Released
  }

  pub fn was_unpressed(&self, key: VirtualKeyCode) -> bool {
    self.key_prev(key) == ElementState::Released
  }

  pub fn is_clicked(&self, button: MouseButton) -> bool {
    self.button(button) == ElementState::Pressed
  }

  pub fn was_clicked(&self, button: MouseButton) -> bool {
    self.button_prev(button) == ElementState::Pressed
  }

  pub fn is_unclicked(&self, button: MouseButton) -> bool {
    self.button(button) == ElementState::Released
  }

  pub fn was_unclicked(&self, button: MouseButton) -> bool {
    self.button_prev(button) == ElementState::Released
  }
}

#[allow(dead_code)]
impl InputState {
  pub fn got_pressed(&self, key: VirtualKeyCode) -> bool {
    self.is_pressed(key) && !self.was_pressed(key)
  }

  pub fn got_unpressed(&self, key: VirtualKeyCode) -> bool {
    self.is_unpressed(key) && !self.was_unpressed(key)
  }

  pub fn got_clicked(&self, button: MouseButton) -> bool {
    self.is_clicked(button) && !self.was_clicked(button)
  }

  pub fn got_unclicked(&self, button: MouseButton) -> bool {
    self.is_unclicked(button) && !self.was_unclicked(button)
  }

  pub fn cursor_screen_pixel_difference(&self) -> Option<ScreenPixelVector> {
    if let Some((prev, curr)) = self
      .prev
      .cursor_pos
      .as_ref()
      .map(|p| p.screen_pixel)
      .zip(self.curr.cursor_pos.as_ref().map(|p| p.screen_pixel))
    {
      Some(curr - prev)
    } else {
      None
    }
  }

  pub fn cursor_screen_norm_difference(&self) -> Option<ScreenNormVector> {
    if let Some((prev, curr)) = self
      .prev
      .cursor_pos
      .as_ref()
      .map(|p| p.screen_norm)
      .zip(self.curr.cursor_pos.as_ref().map(|p| p.screen_norm))
    {
      Some(curr - prev)
    } else {
      None
    }
  }

  pub fn cursor_canvas_difference(&self) -> Option<CanvasVector> {
    if let Some((prev, curr)) = self
      .prev
      .cursor_pos
      .as_ref()
      .map(|p| p.canvas)
      .zip(self.curr.cursor_pos.as_ref().map(|p| p.canvas))
    {
      Some(curr - prev)
    } else {
      None
    }
  }

  pub fn cursor_screen_distance_sqr(&self) -> Option<ScreenPixelUnit> {
    self
      .cursor_screen_pixel_difference()
      .map(|d| d.magnitude_squared())
  }
}

#[derive(Debug)]
pub struct TouchMovement {
  pub center: PointInSpaces,
  pub translation: VectorInSpaces,
  pub rotation: f32,
  pub scale: f32,
}

#[allow(dead_code)]
impl InputState {
  pub fn reset(&mut self) {
    self.prev = self.curr.clone();
    self.mouse_scroll_delta = None;
  }

  pub fn handle_event(&mut self, event: &WindowEvent, window: &Window, camera_screen: &Camera) {
    let store = &mut self.curr;
    match event {
      WindowEvent::MouseInput { state, button, .. } => {
        match state {
          ElementState::Pressed => {
            store.clicked.insert(*button);
          }
          ElementState::Released => {
            store.clicked.remove(button);
            self.cursor_pos_left_clicked = None;
          }
        };
      }
      WindowEvent::KeyboardInput {
        input:
          winit::event::KeyboardInput {
            state,
            virtual_keycode: Some(key),
            ..
          },
        ..
      } => {
        match state {
          ElementState::Pressed => store.pressed.insert(*key),
          ElementState::Released => store.pressed.remove(key),
        };
      }
      WindowEvent::ModifiersChanged(modifiers) => {
        store.modifiers = *modifiers;
      }
      WindowEvent::CursorLeft { .. } => self.curr.cursor_pos = None,
      WindowEvent::CursorMoved {
        position: physical_position,
        ..
      } => {
        self.curr.cursor_pos = Some(PointInSpaces::from_window_physical(
          *physical_position,
          window,
          camera_screen,
        ));

        if self.is_clicked(MouseButton::Left) && self.cursor_pos_left_clicked.is_none() {
          self.cursor_pos_left_clicked = self.curr.cursor_pos.clone();
        }
      }

      WindowEvent::MouseWheel { delta, .. } => {
        const LINE_DELTA: f32 = 10.0;
        let screen_pixel = match *delta {
          MouseScrollDelta::LineDelta(mut x, mut y) => {
            x *= LINE_DELTA;
            y *= LINE_DELTA;
            ScreenPixelVector::new(x.into(), y.into())
          }
          MouseScrollDelta::PixelDelta(delta) => {
            let delta = delta.to_logical(window.scale_factor());
            ScreenPixelVector::from_window_logical(delta)
          }
        };
        let screen_norm = ScreenNormVector::from_screen_pixel(screen_pixel, camera_screen);
        let canvas = CanvasVector::from_screen_pixel(screen_pixel, camera_screen);
        self.mouse_scroll_delta = Some(VectorInSpaces {
          screen_pixel,
          screen_norm,
          canvas,
        });
      }
      WindowEvent::Touch(winit::event::Touch {
        phase,
        location,
        id,
        ..
      }) => {
        use winit::event::TouchPhase;
        let position = PointInSpaces::from_window_physical(*location, window, camera_screen);
        let touch = Touch { position };
        match phase {
          TouchPhase::Started => {
            store.touches.insert(*id, touch);
          }
          TouchPhase::Moved => *store.touches.get_mut(id).unwrap() = touch,
          TouchPhase::Ended => assert!(store.touches.remove(id).is_some()),
          TouchPhase::Cancelled => assert!(store.touches.remove(id).is_some()),
        }
      }
      _ => {}
    };
  }

  pub fn update(&mut self, camera_screen: &Camera) {
    self.curr.update(camera_screen);
    self.multi_touch_movement = self.compute_touch_movement(camera_screen);
  }

  fn compute_touch_movement(&mut self, camera_screen: &Camera) -> Option<TouchMovement> {
    let prev = self.prev.multi_touch.as_ref()?;
    let curr = self.curr.multi_touch.as_ref()?;
    let translation = curr.avg_pos.screen_pixel - prev.avg_pos.screen_pixel;
    let translation = VectorInSpaces::from_screen_pixel(translation, camera_screen);
    let rotation = (curr.heading - prev.heading).rem_euclid(std::f32::consts::TAU);
    let scale = curr.avg_dist / prev.avg_dist;
    let scale = scale.into();

    let center = curr.avg_pos.clone();

    Some(TouchMovement {
      center,
      translation,
      rotation,
      scale,
    })
  }
}

impl InputsSnapshot {
  fn update(&mut self, camera_screen: &Camera) {
    if let Some(cursor_pos) = &mut self.cursor_pos {
      cursor_pos.canvas = CanvasPoint::from_screen_pixel(cursor_pos.screen_pixel, camera_screen);
    }
    self.multi_touch = self.compute_multi_touch(camera_screen);
  }

  fn compute_multi_touch(&self, camera_screen: &Camera) -> Option<MultiTouch> {
    if self.touches.len() == 2 {
      let recip = 1.0 / 2.0;

      let [t0, t1] = {
        let mut ts = self
          .touches
          .values()
          .map(|t| t.position.screen_pixel)
          .take(2);
        [ts.next().unwrap(), ts.next().unwrap()]
      };

      let avg_pos = ScreenPixelPoint::from((t0.coords + t1.coords).scale(recip.into()));
      let avg_pos = PointInSpaces::from_screen_pixel(avg_pos, camera_screen);

      let mut avg_dist = ScreenPixelUnit::new(0.0);
      for t in [t0, t1] {
        avg_dist += (avg_pos.screen_pixel - t).magnitude();
      }
      avg_dist *= recip.into();

      let diff = (t1 - t0).cast::<f32>();
      let heading = diff.y.atan2(diff.x);

      Some(MultiTouch {
        avg_pos,
        avg_dist,
        heading,
      })
    } else {
      None
    }
  }
}

type TouchId = u64;

#[derive(Clone, Debug)]
pub struct Touch {
  pub position: PointInSpaces,
}

#[derive(Debug, Clone)]
pub struct MultiTouch {
  avg_pos: PointInSpaces,
  avg_dist: ScreenPixelUnit,
  heading: f32,
}
