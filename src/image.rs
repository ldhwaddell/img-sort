use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct Image {
    pub name: String,
    pub path: PathBuf,
}

impl Image {
    pub fn new(path: PathBuf, name: String) -> Self {
        Image { path, name }
    }
}
