use std::{fs::read_to_string, path::PathBuf};

use crate::err;

pub trait ValidID {
    fn is_id(&self) -> bool;
}

impl ValidID for char {
    fn is_id(&self) -> bool {
        *self == '_' || self.is_ascii_alphanumeric()
    }
}

pub fn read_file(path: &PathBuf) -> String {
    match read_to_string(&path) {
        Ok(v) => v,
        _ => err!(
            "couldn't find `{}` in `{}`",
            path.file_name().unwrap().to_string_lossy(),
            path.parent().unwrap().display()
        ),
    }
}
