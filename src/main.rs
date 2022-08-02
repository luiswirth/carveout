#![allow(clippy::single_match)]

mod canvas;
mod gfx;
mod ui;
mod util;

use crate::{canvas::Canvas, gfx::Gfx, ui::Ui};

use winit::{event_loop::ControlFlow, window::Window};
pub type CustomEvent = ();
pub type Event<'a> = winit::event::Event<'a, CustomEvent>;
pub type EventLoop = winit::event_loop::EventLoop<CustomEvent>;

pub struct Application {
  event_loop: Option<EventLoop>,
  window: Window,

  gfx: Gfx,
  ui: Ui,

  canvas: Canvas,
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
    let mut ui = Ui::init(&event_loop, gfx.wgpu().device());

    let canvas = Canvas::init(gfx.wgpu().device(), ui.renderer_mut());

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

    self.gfx.handle_event(&event);
    self.ui.handle_event(&event);
    self.canvas.handle_event(&event, &self.window);

    match event {
      Event::NewEvents(_) => {}
      Event::MainEventsCleared => {
        self.update();

        // TODO: do not always request redraw
        self.window.request_redraw();
      }
      Event::RedrawRequested(_) => {
        self.render();
      }
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        window_id,
      } if window_id == self.window.id() => *control_flow = ControlFlow::Exit,

      Event::RedrawEventsCleared => {}
      Event::Suspended => {}
      Event::Resumed => {}
      Event::LoopDestroyed => {}

      _ => {}
    }
  }

  fn update(&mut self) {}

  fn render(&mut self) {
    self.gfx.render(|encoder, rt, wgpu| {
      self.canvas.render(wgpu.device(), wgpu.queue(), encoder);
      self.ui.render(
        &self.window,
        wgpu.device(),
        wgpu.queue(),
        encoder,
        rt,
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
