use crate::content::{protocol::Protocol, Content};

use serde::{Deserialize, Serialize};
use std::{
  borrow::Cow,
  ffi::OsStr,
  fs,
  io::{Read, Write},
  path::Path,
};

#[derive(Serialize, Deserialize)]
pub struct Savefile {
  pub content: Content,
  pub protocol: Protocol,
}

pub fn load(file_path: &Path) -> Savefile {
  let mut file = fs::File::open(file_path).unwrap();
  let mut data_string = String::new();
  file.read_to_string(&mut data_string).unwrap();
  ron::from_str(&data_string).unwrap()
}

pub fn save<'a>(savefile: &Savefile, file_path: impl Into<Cow<'a, Path>>) {
  let mut file_path = file_path.into();
  if file_path.extension() != Some(OsStr::new("co")) {
    file_path.to_mut().set_extension("co");
  }
  let pretty_config = ron::ser::PrettyConfig::default();
  let data_string = ron::ser::to_string_pretty(savefile, pretty_config).unwrap();
  let mut file = fs::File::create(file_path).unwrap();
  file.write_all(data_string.as_bytes()).unwrap();
}
