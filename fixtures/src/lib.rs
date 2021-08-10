#[macro_use]
extern crate lazy_static;

use std::path::{Path, PathBuf};

pub mod models;

lazy_static! {
    pub static ref DATA_PATH: PathBuf = Path::new(module_path!()).join("data");
}
