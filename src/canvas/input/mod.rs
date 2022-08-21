mod pen;

use self::pen::PenInputHandler;

use super::{
  content::{CanvasContent, RemoveStrokeCommand},
  gfx::CameraWithScreen,
  protocol::ProtocolManager,
  space::*,
  stroke::{StrokeId, StrokeManager},
  tool::{ToolConfig, ToolEnum},
};

use crate::Event;

use parry2d::query::PointQuery;
use winit::{
  event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
  window::Window,
};

#[derive(Default)]
pub struct InputHandler {
  pen_handler: PenInputHandler,
  mouse_clicked: bool,
  click_cursor_pos: Option<ScreenPixelPoint>,
  last_cursor_pos: Option<ScreenPixelPoint>,
}

impl InputHandler {
  pub fn handle_event(
    &mut self,
    event: &Event,
    window: &winit::window::Window,
    camera_screen: &mut CameraWithScreen,
    tool_config: &ToolConfig,
    content_commander: &mut ProtocolManager,
    canvas_content: &mut CanvasContent,
    stroke_manager: &StrokeManager,
  ) {
    if let Event::WindowEvent { event, window_id } = event {
      assert_eq!(*window_id, window.id());

      match tool_config.selected {
        ToolEnum::Pen => self.pen_handler.handle_event(
          event,
          window,
          camera_screen,
          &tool_config.pen,
          content_commander,
          canvas_content,
        ),
        ToolEnum::Eraser => self.handle_eraser_tool_event(
          event,
          window,
          camera_screen,
          content_commander,
          canvas_content,
          stroke_manager,
        ),
        ToolEnum::Translate => self.handle_translate_tool_event(event, window, camera_screen),
        ToolEnum::Rotate => self.handle_rotate_tool_event(event, window, camera_screen),
        ToolEnum::Scale => self.handle_scale_tool_event(event, window, camera_screen),
      }
      self.handle_scale_event(event, window, camera_screen);
    }
  }

  pub fn handle_eraser_tool_event(
    &mut self,
    event: &WindowEvent,
    window: &Window,
    camera_screen: &mut CameraWithScreen,
    undo_tree: &mut ProtocolManager,
    canvas_content: &mut CanvasContent,
    stroke_manager: &StrokeManager,
  ) {
    match event {
      WindowEvent::CursorMoved { position, .. } => {
        if !self.mouse_clicked {
          return;
        }
        let pos = position.to_logical(window.scale_factor());
        let pos = match ScreenPixelPoint::try_from_window_logical(pos, camera_screen) {
          Some(p) => p,
          None => return,
        };
        let pos = CanvasPoint::from_screen(pos, camera_screen);
        let remove_list: Vec<StrokeId> = canvas_content
          .persistent()
          .strokes()
          .values()
          .filter(|s| {
            let mesh = stroke_manager
              .data()
              .parry_meshes
              .get(&s.id())
              .expect("No parry data.");
            mesh.contains_point(&na::Isometry2::default(), &pos.cast())
          })
          .map(|s| s.id())
          .collect();

        for id in remove_list {
          undo_tree.do_it(Box::new(RemoveStrokeCommand::new(id)))
        }
      }

      WindowEvent::MouseInput { state, button, .. } => {
        if *button == MouseButton::Left {
          match state {
            ElementState::Pressed => self.mouse_clicked = true,
            ElementState::Released => self.mouse_clicked = false,
          }
        }
      }
      _ => {}
    }
  }

  fn handle_scale_event(
    &mut self,
    event: &WindowEvent,
    window: &Window,
    camera_screen: &mut CameraWithScreen,
  ) {
    match event {
      WindowEvent::CursorMoved { position, .. } => {
        let pos = position.to_logical(window.scale_factor());
        let pos = match ScreenPixelPoint::try_from_window_logical(pos, camera_screen) {
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
            let center = CanvasPoint::from_screen(cursor_pos, camera_screen);
            camera_screen
              .camera_mut()
              .scale_with_center(scale_factor, center);
          }
          None => {}
        }
      }
      _ => {}
    }
  }

  pub fn handle_translate_tool_event(
    &mut self,
    event: &WindowEvent,
    window: &Window,
    camera_screen: &mut CameraWithScreen,
  ) {
    match event {
      WindowEvent::CursorMoved { position, .. } => {
        if !self.mouse_clicked {
          return;
        }
        let pos = position.to_logical(window.scale_factor());
        let pos = match ScreenPixelPoint::try_from_window_logical(pos, camera_screen) {
          Some(p) => p,
          None => return,
        };

        match self.last_cursor_pos {
          None => self.last_cursor_pos = Some(pos),
          Some(last_pos) => {
            let diff = pos - last_pos;
            let diff = CanvasVector::from_screen(diff, camera_screen);
            camera_screen.camera_mut().position -= diff;
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

  pub fn handle_rotate_tool_event(
    &mut self,
    event: &WindowEvent,
    window: &Window,
    camera_screen: &mut CameraWithScreen,
  ) {
    match event {
      WindowEvent::CursorMoved { position, .. } => {
        if !self.mouse_clicked {
          return;
        }
        let pos = position.to_logical(window.scale_factor());
        let pos = match ScreenPixelPoint::try_from_window_logical(pos, camera_screen) {
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
            let diff = ScreenNormalizedVector::from_pixel(diff, camera_screen);
            let rotation = diff.x.0 * std::f32::consts::TAU;

            let center = CanvasPoint::from_screen(self.click_cursor_pos.unwrap(), camera_screen);
            camera_screen
              .camera_mut()
              .rotate_with_center(rotation, center);

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

  pub fn handle_scale_tool_event(
    &mut self,
    event: &WindowEvent,
    window: &Window,
    camera_screen: &mut CameraWithScreen,
  ) {
    match event {
      WindowEvent::CursorMoved { position, .. } => {
        if !self.mouse_clicked {
          return;
        }
        let pos = position.to_logical(window.scale_factor());
        let pos = match ScreenPixelPoint::try_from_window_logical(pos, camera_screen) {
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
            let diff = ScreenNormalizedVector::from_pixel(diff, camera_screen);
            let scale_factor = 1.0 + diff.y.0;

            let center = CanvasPoint::from_screen(self.click_cursor_pos.unwrap(), camera_screen);
            camera_screen
              .camera_mut()
              .scale_with_center(scale_factor, center);

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
