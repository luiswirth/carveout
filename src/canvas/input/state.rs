use crate::canvas::{
  gfx::CameraWithScreen,
  space::{
    CanvasPoint, CanvasPointExt, CanvasVector, CanvasVectorExt, ScreenNormPoint,
    ScreenNormPointExt, ScreenNormVector, ScreenNormVectorExt, ScreenPixelPoint,
    ScreenPixelPointExt, ScreenPixelUnit, ScreenPixelVector, ScreenPixelVectorExt,
  },
};

use std::collections::HashSet;
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

  pub cursor_pos_left_clicked: Option<CursorPos>,
  pub mouse_scroll_delta: Option<ScrollDelta>,
}

#[derive(Default, Clone)]
pub struct InputsSnapshot {
  pub pressed: HashSet<VirtualKeyCode>,
  pub clicked: HashSet<MouseButton>,
  pub modifiers: ModifiersState,
  pub cursor_pos: Option<CursorPos>,
}

#[derive(Clone)]
pub struct CursorPos {
  pub screen_pixel: ScreenPixelPoint,
  pub screen_norm: ScreenNormPoint,
  pub canvas: CanvasPoint,
}

pub struct ScrollDelta {
  pub screen_pixel: ScreenPixelVector,
  pub screen_norm: ScreenNormVector,
  pub canvas: CanvasVector,
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

#[allow(dead_code)]
impl InputState {
  pub fn update(&mut self, camera_screen: &CameraWithScreen) {
    self.prev = self.curr.clone();
    self.mouse_scroll_delta = None;

    if let Some(cursor_pos) = &mut self.curr.cursor_pos {
      cursor_pos.canvas = CanvasPoint::from_screen_pixel(cursor_pos.screen_pixel, camera_screen);
    }
  }

  pub fn handle_event(
    &mut self,
    event: &WindowEvent,
    window: &Window,
    camera_screen: &CameraWithScreen,
  ) {
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
        position: logical_pos,
        ..
      } => {
        let screen_pixel = ScreenPixelPoint::try_from_window_logical(
          logical_pos.to_logical(window.scale_factor()),
          camera_screen,
        );
        let screen_norm =
          screen_pixel.map(|p| ScreenNormPoint::from_screen_pixel(p, camera_screen));
        let canvas = screen_pixel.map(|p| CanvasPoint::from_screen_pixel(p, camera_screen));

        self.curr.cursor_pos =
          screen_pixel
            .zip(screen_norm)
            .zip(canvas)
            .map(|((screen_pixel, screen_norm), canvas)| CursorPos {
              screen_pixel,
              screen_norm,
              canvas,
            });

        if self.is_clicked(MouseButton::Left) && self.cursor_pos_left_clicked.is_none() {
          self.cursor_pos_left_clicked = self.curr.cursor_pos.clone();
        }
      }

      WindowEvent::MouseWheel { delta, .. } => {
        const LINE_DELTA: f32 = 50.0;
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
        self.mouse_scroll_delta = Some(ScrollDelta {
          screen_pixel,
          screen_norm,
          canvas,
        });
      }
      _ => {}
    };
  }
}
