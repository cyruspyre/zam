use std::{
    fs::read_to_string,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use crate::{err, zam::block::Hoistable};

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

#[derive(Debug, Clone)]
pub struct Span<T> {
    pub rng: [usize; 2],
    pub data: T,
}

pub trait ToSpan {
    fn span(self, rng: [usize; 2]) -> Span<Self>
    where
        Self: Sized,
    {
        Span { rng, data: self }
    }
}

impl ToSpan for Hoistable {}

impl<T> Deref for Span<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for Span<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
