use carveout::Application;

fn main() {
  let future = async {
    let app = Application::init().await;
    app.run();
  };
  futures::executor::block_on(future);
}
