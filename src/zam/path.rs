use std::{
    fmt::{Debug, Display, Formatter, Result},
    ops::{Deref, DerefMut},
};

use crate::misc::Ref;

#[derive(Default, Clone, Eq, Hash, PartialEq)]
pub struct ZamPath(pub Vec<Ref<String>>);

impl Deref for ZamPath {
    type Target = Vec<Ref<String>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ZamPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for ZamPath {
    fn fmt(&self, f: &mut Formatter) -> Result {
        super::misc::display(&self.0, f)
    }
}

impl Debug for ZamPath {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write_str(&format!("\"{self}\""))
    }
}
