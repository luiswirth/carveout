use crate::canvas::content::PersistentContent;

use serde::{Deserialize, Serialize};
use std::{
  fs,
  io::{Read, Write},
};

#[derive(Serialize, Deserialize)]
pub struct Savefile {
  pub content: PersistentContent,
}

pub fn load() -> Option<Savefile> {
  let home_dir = dirs::home_dir().unwrap();
  let file_path = rfd::FileDialog::new()
    .add_filter("carveout", &["co"])
    .set_directory(home_dir)
    .pick_file();

  if let Some(file_path) = file_path {
    let mut file = fs::File::open(file_path).unwrap();

    let mut data_string = String::new();
    file.read_to_string(&mut data_string).unwrap();
    let savefile = ron::from_str(&data_string).unwrap();

    Some(savefile)
  } else {
    None
  }
}

pub fn save(savefile: &Savefile) {
  let home_dir = dirs::home_dir().unwrap();
  let file_path = rfd::FileDialog::new()
    .add_filter("carveout", &["co"])
    .set_directory(home_dir)
    .save_file();
  if let Some(mut file_path) = file_path {
    match file_path.extension() {
      Some(ext) if ext == "co" => true,
      _ => file_path.set_extension("co"),
    };

    let data_string = ron::to_string(savefile).unwrap();

    let mut file = fs::File::create(file_path).unwrap();
    file.write_all(data_string.as_bytes()).unwrap();
  }
}
