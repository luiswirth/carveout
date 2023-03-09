use std::collections::{HashMap, HashSet};
use winit::event::{
  ElementState, ModifiersState, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
};

use crate::spaces::{Space, SpaceManager};

#[derive(Default)]
pub struct InputManager {
  pub prev: InputsSnapshot,
  pub curr: InputsSnapshot,

  pub cursor_pos_screen_logical_left_clicked: Option<na::Point2<f32>>,
  pub mouse_scroll_delta_logical: Option<na::Vector2<f32>>,
  pub multi_touch_movement: Option<TouchMovement>,
}

#[derive(Default, Clone)]
pub struct InputsSnapshot {
  pub pressed: HashSet<VirtualKeyCode>,
  pub clicked: HashSet<MouseButton>,
  pub touches: HashMap<TouchId, Touch>,
  pub multi_touch: Option<MultiTouch>,
  pub modifiers: ModifiersState,
  pub cursor_pos_screen_logical: Option<na::Point2<f32>>,
}

#[allow(dead_code)]
impl InputManager {
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
impl InputManager {
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
impl InputManager {
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

  pub fn cursor_screen_logical_difference(&self) -> Option<na::Vector2<f32>> {
    if let Some((prev, curr)) = self
      .prev
      .cursor_pos_screen_logical
      .as_ref()
      .zip(self.curr.cursor_pos_screen_logical.as_ref())
    {
      Some(curr - prev)
    } else {
      None
    }
  }
}

#[derive(Debug)]
pub struct TouchMovement {
  pub center_screen_logical: na::Point2<f32>,
  pub translation_screen_logical: na::Vector2<f32>,
  pub rotation: f32,
  pub scale: f32,
}

#[allow(dead_code)]
impl InputManager {
  pub fn reset(&mut self) {
    self.prev = self.curr.clone();
    self.mouse_scroll_delta_logical = None;
  }

  pub fn handle_event(&mut self, event: &WindowEvent, spaces: &SpaceManager) {
    dbg!(event);
    let store = &mut self.curr;
    match event {
      WindowEvent::MouseInput { state, button, .. } => {
        match state {
          ElementState::Pressed => {
            store.clicked.insert(*button);
          }
          ElementState::Released => {
            store.clicked.remove(button);
            self.cursor_pos_screen_logical_left_clicked = None;
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
      WindowEvent::CursorLeft { .. } => self.curr.cursor_pos_screen_logical = None,
      WindowEvent::CursorMoved { position, .. } => {
        self.curr.cursor_pos_screen_logical = Some(spaces.transform_point(
          na::point![position.x as f32, position.y as f32],
          Space::WindowPhysical,
          Space::ScreenLogical,
        ));

        if self.is_clicked(MouseButton::Left)
          && self.cursor_pos_screen_logical_left_clicked.is_none()
        {
          self.cursor_pos_screen_logical_left_clicked = self.curr.cursor_pos_screen_logical;
        }
      }

      WindowEvent::MouseWheel { delta, .. } => {
        const LINE_DELTA: f32 = 10.0;
        let window_physical = match *delta {
          MouseScrollDelta::LineDelta(mut x, mut y) => {
            x *= LINE_DELTA;
            y *= LINE_DELTA;
            na::vector![x, y]
          }
          MouseScrollDelta::PixelDelta(delta) => {
            na::vector![delta.x as f32, delta.y as f32]
          }
        };
        self.mouse_scroll_delta_logical = Some(spaces.transform_vector(
          window_physical,
          Space::WindowPhysical,
          Space::ScreenLogical,
        ));
      }
      WindowEvent::Touch(winit::event::Touch {
        phase,
        location,
        id,
        ..
      }) => {
        use winit::event::TouchPhase;
        let location = na::point![location.x as f32, location.y as f32];
        let position_screen_logical =
          spaces.transform_point(location, Space::WindowPhysical, Space::ScreenLogical);
        let touch = Touch {
          position_screen_logical,
        };
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

  pub fn update(&mut self) {
    self.curr.multi_touch = self.curr.compute_multi_touch();
    self.multi_touch_movement = self.compute_touch_movement();
  }

  fn compute_touch_movement(&mut self) -> Option<TouchMovement> {
    let prev = self.prev.multi_touch.as_ref()?;
    let curr = self.curr.multi_touch.as_ref()?;
    let translation_screen_logical = curr.avg_pos_screen_logical - prev.avg_pos_screen_logical;
    let rotation = (curr.heading - prev.heading).rem_euclid(std::f32::consts::TAU);
    let scale = curr.avg_dist_screen_logical / prev.avg_dist_screen_logical;

    let center_screen_logical = curr.avg_pos_screen_logical;

    Some(TouchMovement {
      center_screen_logical,
      translation_screen_logical,
      rotation,
      scale,
    })
  }
}

impl InputsSnapshot {
  fn compute_multi_touch(&self) -> Option<MultiTouch> {
    if self.touches.len() == 2 {
      let [t0, t1] = {
        let mut ts = self
          .touches
          .values()
          .map(|t| t.position_screen_logical)
          .take(2);
        [ts.next().unwrap(), ts.next().unwrap()]
      };

      let avg_pos = na::Point2::from((t0.coords + t1.coords) / 2.0);
      let avg_dist = ((avg_pos - t0).magnitude() + (avg_pos - t1).magnitude()) / 2.0;
      let diff = t1 - t0;
      let heading = diff.y.atan2(diff.x);

      Some(MultiTouch {
        avg_pos_screen_logical: avg_pos,
        avg_dist_screen_logical: avg_dist,
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
  pub position_screen_logical: na::Point2<f32>,
}

#[derive(Debug, Clone)]
pub struct MultiTouch {
  avg_pos_screen_logical: na::Point2<f32>,
  avg_dist_screen_logical: f32,
  heading: f32,
}
