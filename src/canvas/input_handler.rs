use super::{
  tool::{ToolConfig, ToolEnum},
  CanvasPortal,
};

use crate::{util::space::*, Event};

use winit::{
  event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
  window::Window,
};

#[derive(Default)]
pub struct InputHandler {
  mouse_clicked: bool,
  click_cursor_pos: Option<PortalPoint>,
  last_cursor_pos: Option<PortalPoint>,
}

impl InputHandler {
  pub fn handle_event(
    &mut self,
    event: &crate::Event,
    window: &winit::window::Window,
    portal: &mut CanvasPortal,
    tool_config: &ToolConfig,
    stroke_manager: &mut super::stroke::StrokeManager,
  ) {
    match tool_config.selected {
      ToolEnum::Pen => stroke_manager.handle_event(event, window, portal, &tool_config.pen),
      ToolEnum::Translate => self.handle_translate_tool_event(event, window, portal),
      ToolEnum::Rotate => self.handle_rotate_tool_event(event, window, portal),
      ToolEnum::Scale => self.handle_scale_tool_event(event, window, portal),
    }
    self.handle_scale_event(event, window, portal);
  }

  fn handle_scale_event(
    &mut self,
    event: &crate::Event,
    window: &Window,
    portal: &mut CanvasPortal,
  ) {
    if let Event::WindowEvent { event, window_id } = event {
      match event {
        WindowEvent::CursorMoved { position, .. } => {
          assert_eq!(window.id(), *window_id);

          let pos = WindowPhysicalPoint::from_underlying(*position);
          let pos = WindowLogicalPoint::from_physical(pos, window.scale_factor() as f32);
          let pos = match PortalPoint::try_from_window_logical(pos, portal) {
            Some(p) => p,
            None => return,
          };
          self.last_cursor_pos = Some(pos);
        }
        WindowEvent::MouseWheel { delta, .. } => {
          let scale_factor = match delta {
            MouseScrollDelta::LineDelta(_x, y) => 1.0 - y / 100.0,
            MouseScrollDelta::PixelDelta(_) => unimplemented!(),
          };
          match self.last_cursor_pos {
            Some(cursor_pos) => portal.scale_with_center(scale_factor, cursor_pos),
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
    portal: &mut CanvasPortal,
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
          let pos = match PortalPoint::try_from_window_logical(pos, portal) {
            Some(p) => p,
            None => return,
          };

          match self.last_cursor_pos {
            None => self.last_cursor_pos = Some(pos),
            Some(last_pos) => {
              let diff = pos - last_pos;
              let diff = CanvasVector::from_portal(diff, portal);
              portal.position_canvas -= diff;
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

  pub fn handle_rotate_tool_event(
    &mut self,
    event: &Event,
    window: &Window,
    portal: &mut CanvasPortal,
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
          let pos = match PortalPoint::try_from_window_logical(pos, portal) {
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
              let rotation = diff.x * 10.0;

              portal.rotate_with_center(rotation, self.click_cursor_pos.unwrap());

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

  pub fn handle_scale_tool_event(
    &mut self,
    event: &Event,
    window: &Window,
    portal: &mut CanvasPortal,
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
          let pos = match PortalPoint::try_from_window_logical(pos, portal) {
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
              let scale_factor = 1.0 + diff.y;

              portal.scale_with_center(scale_factor, self.click_cursor_pos.unwrap());

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
