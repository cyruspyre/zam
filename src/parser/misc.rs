use std::{fs::read_to_string, ops::Deref, path::PathBuf};

use crate::err;

#[derive(Debug)]
pub struct Context<C, D> {
    pub ctx: C,
    pub data: D,
}

impl<C, D> Deref for Context<C, D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

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

pub trait Either<T> {
    fn either(self) -> T;
}

impl<T> Either<T> for Result<T, T> {
    fn either(self) -> T {
        match self {
            Ok(v) => v,
            Err(e) => e,
        }
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
