use winit::{
  event::{ElementState, MouseButton},
  window::Window,
};

use crate::{util::space::*, Event};
use winit::event::{MouseScrollDelta, WindowEvent};

use super::{
  tool::{ToolConfig, ToolEnum},
  CanvasViewport,
};

#[derive(Default)]
pub struct InputHandler {
  mouse_clicked: bool,
  click_cursor_pos: Option<CanvasViewportPoint>,
  last_cursor_pos: Option<CanvasViewportPoint>,
}

impl InputHandler {
  pub fn handle_event(
    &mut self,
    event: &crate::Event,
    window: &winit::window::Window,
    viewport: &mut CanvasViewport,
    viewport_box: WindowLogicalBox,
    tool_config: &ToolConfig,
    stroke_manager: &mut super::stroke::StrokeManager,
  ) {
    match tool_config.selected {
      ToolEnum::Pen => stroke_manager.handle_event(event, window, viewport_box, &tool_config.pen),
      ToolEnum::Translate => {
        self.handle_translate_tool_event(event, window, viewport_box, viewport)
      }
      ToolEnum::Scale => self.handle_scale_tool_event(event, window, viewport_box, viewport),
    }
    self.handle_scale_event(event, window, viewport_box, viewport);
  }

  fn handle_scale_event(
    &mut self,
    event: &crate::Event,
    window: &Window,
    viewport_box: WindowLogicalBox,
    viewport: &mut CanvasViewport,
  ) {
    if let Event::WindowEvent { event, window_id } = event {
      match event {
        WindowEvent::CursorMoved { position, .. } => {
          assert_eq!(window.id(), *window_id);

          let pos = WindowPhysicalPoint::from_underlying(*position);
          let pos = WindowLogicalPoint::from_physical(pos, window.scale_factor() as f32);
          let pos = match CanvasViewportPoint::from_window_logical(pos, viewport_box) {
            Some(p) => p,
            None => return,
          };
          self.last_cursor_pos = Some(pos);
        }
        WindowEvent::MouseWheel { delta, .. } => {
          let scale_factor = match delta {
            MouseScrollDelta::LineDelta(_x, y) => 1.0 + y / 100.0,
            MouseScrollDelta::PixelDelta(_) => unimplemented!(),
          };
          match self.last_cursor_pos {
            Some(cursor_pos) => {
              // scale with cursor as origin
              viewport.transform = viewport
                .transform
                .then_translate(-cursor_pos.to_vector())
                .then_scale(scale_factor, scale_factor)
                .then_translate(cursor_pos.to_vector());
            }
            None => {}
          }
        }
        _ => {}
      }
    }
  }

  pub fn handle_translate_tool_event(
    &mut self,
    event: &Event,
    window: &Window,
    viewport_box: WindowLogicalBox,
    viewport: &mut CanvasViewport,
  ) {
    if let Event::WindowEvent { event, window_id } = event {
      match event {
        WindowEvent::CursorMoved { position, .. } => {
          assert_eq!(window.id(), *window_id);
          if !self.mouse_clicked {
            return;
          }
          let pos = WindowPhysicalPoint::from_underlying(*position);
          let pos = WindowLogicalPoint::from_physical(pos, window.scale_factor() as f32);
          let pos = match CanvasViewportPoint::from_window_logical(pos, viewport_box) {
            Some(p) => p,
            None => return,
          };

          match self.last_cursor_pos {
            None => self.last_cursor_pos = Some(pos),
            Some(last_pos) => {
              let diff = pos - last_pos;
              viewport.transform = viewport.transform.then_translate(diff);
              self.last_cursor_pos = Some(pos);
            }
          }
        }
        WindowEvent::MouseInput { state, button, .. } => {
          if *button == MouseButton::Left {
            match state {
              ElementState::Pressed => {
                self.mouse_clicked = true;
                self.last_cursor_pos = None;
              }
              ElementState::Released => {
                self.mouse_clicked = false;
                self.last_cursor_pos = None;
              }
            }
          }
        }
        _ => {}
      }
    }
  }
  pub fn handle_scale_tool_event(
    &mut self,
    event: &Event,
    window: &Window,
    viewport_box: WindowLogicalBox,
    viewport: &mut CanvasViewport,
  ) {
    if let Event::WindowEvent { event, window_id } = event {
      match event {
        WindowEvent::CursorMoved { position, .. } => {
          assert_eq!(window.id(), *window_id);
          if !self.mouse_clicked {
            return;
          }
          let pos = WindowPhysicalPoint::from_underlying(*position);
          let pos = WindowLogicalPoint::from_physical(pos, window.scale_factor() as f32);
          let pos = match CanvasViewportPoint::from_window_logical(pos, viewport_box) {
            Some(p) => p,
            None => return,
          };

          if self.click_cursor_pos.is_none() {
            self.click_cursor_pos = Some(pos);
          }

          match self.last_cursor_pos {
            None => self.last_cursor_pos = Some(pos),
            Some(last_pos) => {
              let diff = pos - last_pos;
              let scale_factor = 1.0 - diff.y;

              // scale with cursor as origin
              viewport.transform = viewport
                .transform
                .then_translate(-self.click_cursor_pos.unwrap().to_vector())
                .then_scale(scale_factor, scale_factor)
                .then_translate(self.click_cursor_pos.unwrap().to_vector());

              self.last_cursor_pos = Some(pos);
            }
          }
        }
        WindowEvent::MouseInput { state, button, .. } => {
          if *button == MouseButton::Left {
            match state {
              ElementState::Pressed => {
                self.mouse_clicked = true;
                self.last_cursor_pos = None;
              }
              ElementState::Released => {
                self.mouse_clicked = false;
                self.last_cursor_pos = None;
                self.click_cursor_pos = None;
              }
            }
          }
        }
        _ => {}
      }
    }
  }
}
