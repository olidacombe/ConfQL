#[macro_use]
extern crate lazy_static;

use std::path::Path;

pub mod models;

lazy_static! {
    static ref DATA_PATH: &'static Path = Path::new(module_path!());
}
