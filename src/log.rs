use crate::util;

use std::{
  fs::{self, File},
  sync::Arc,
};
use tracing::metadata::LevelFilter;
use tracing_subscriber::{filter, prelude::*};

pub fn init_log() {
  let stdout_log = tracing_subscriber::fmt::layer()
    .with_ansi(true)
    .pretty()
    .with_filter(LevelFilter::WARN)
    // TODO: remove this
    .with_filter(filter::filter_fn(|metadata| {
      !metadata.target().ends_with("wayland::seat::pointer")
    }));

  let file_log = create_log_file().map(|file| {
    tracing_subscriber::fmt::layer()
      .with_ansi(false)
      .with_writer(Arc::new(file))
      .with_filter(LevelFilter::INFO)
      // TODO: remove this
      .with_filter(filter::filter_fn(|metadata| {
        !metadata.target().ends_with("wayland::seat::pointer")
      }))
  });

  tracing_subscriber::registry()
    .with(stdout_log)
    .with(file_log)
    .init();
}

fn create_log_file() -> Option<File> {
  let cache_dir = util::APP_DIRS.cache_dir();
  let path = cache_dir.join("logs");
  fs::create_dir_all(path.clone()).ok()?;
  let file_name = format!("{}.log", chrono::Local::now().format("%Y%m%dT%H%M%S"));
  File::create(path.join(file_name)).ok()
}