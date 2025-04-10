use std::{fs::read_to_string, path::PathBuf};

use crate::err;

pub trait CharExt {
    fn is_id(&self) -> bool;
    fn is_quote(&self) -> bool;
}

impl CharExt for char {
    fn is_id(&self) -> bool {
        *self == '_' || self.is_ascii_alphanumeric()
    }

    fn is_quote(&self) -> bool {
        matches!(self, '"' | '\'')
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
