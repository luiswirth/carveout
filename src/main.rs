#![allow(clippy::single_match)]

extern crate nalgebra as na;

mod canvas;
mod gfx;
mod ui;
mod util;

use crate::{canvas::CanvasManager, gfx::Gfx, ui::Ui};

use winit::{event_loop::ControlFlow, window::Window};

pub type CustomEvent = ();
pub type Event<'a> = winit::event::Event<'a, CustomEvent>;
pub type EventLoop = winit::event_loop::EventLoop<CustomEvent>;

pub struct Application {
  event_loop: Option<EventLoop>,
  window: Window,
  gfx: Gfx,
  ui: Ui,

  canvas: CanvasManager,
}

impl Application {
  pub async fn init() -> Application {
    util::init_log();

    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
      .with_title(env!("CARGO_PKG_NAME"))
      .build(&event_loop)
      .expect("Fatal error: Failed to create winit window.");
    let gfx = Gfx::init(&window).await;
    let ui = Ui::init(&event_loop, gfx.wgpu().device());
    let canvas = CanvasManager::init(
      gfx.wgpu().device(),
      std::rc::Rc::clone(ui.canvas().screen()),
    );

    Self {
      event_loop: Some(event_loop),
      window,
      gfx,
      ui,

      canvas,
    }
  }

  pub fn run(mut self) {
    self
      .event_loop
      .take()
      .unwrap()
      .run(move |event, _, control_flow| {
        self.handle_event(event, control_flow);
      });
  }

  fn handle_event(&mut self, event: Event<'_>, control_flow: &mut ControlFlow) {
    use winit::event::WindowEvent;

    if self.ui.handle_event(&event) && !self.ui.canvas().has_focus() {
      return;
    }
    self.canvas.handle_event(&event, &self.window);

    match event {
      Event::NewEvents(_) => {}
      Event::MainEventsCleared => {
        self.update();
        self.window.request_redraw();
      }
      Event::RedrawRequested(_) => {
        self.render();
      }
      Event::WindowEvent { event, .. } => match event {
        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
        WindowEvent::Resized(new_size) => self.gfx.resize([new_size.width, new_size.height]),
        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
          self
            .gfx
            .resize([new_inner_size.width, new_inner_size.height]);
        }
        _ => {}
      },

      Event::RedrawEventsCleared => {}
      Event::Suspended => {}
      Event::Resumed => {}
      Event::LoopDestroyed => {}

      _ => {}
    }
  }

  fn update(&mut self) {
    self.canvas.update();
  }

  fn render(&mut self) {
    self.gfx.render(|wgpu, encoder, render_target| {
      self.canvas.render(wgpu.device(), wgpu.queue(), encoder);

      self.ui.render(
        &self.window,
        wgpu.device(),
        wgpu.queue(),
        encoder,
        render_target,
        &mut self.canvas,
      );
    });
  }
}

fn main() {
  let future = async {
    let app = Application::init().await;
    app.run();
  };
  futures::executor::block_on(future);
}
