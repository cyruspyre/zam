use std::{
    borrow::Borrow,
    fmt::{Debug, Display, Formatter, Result},
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

use super::Parser;

//pub type Identifier = Span<String>;

#[derive(Clone)]
pub struct Span<T> {
    pub rng: [usize; 2],
    pub data: T,
}

impl Parser {
    pub fn span<T>(&self, data: T) -> Span<T> {
        Span {
            rng: self.rng,
            data,
        }
    }
}

impl<T: Default> Default for Span<T> {
    fn default() -> Self {
        Self {
            rng: [0; 2],
            data: T::default(),
        }
    }
}

impl<T> Borrow<T> for Span<T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<T> Borrow<T> for &Span<T> {
    fn borrow(&self) -> &T {
        &self
    }
}

impl<T> Borrow<T> for &mut Span<T> {
    fn borrow(&self) -> &T {
        &self
    }
}

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

impl<T: Debug> Debug for Span<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if f.alternate() {
            return self.data.fmt(f);
        }

        f.debug_struct("Span")
            .field("rng", &self.rng)
            .field("data", &self.data)
            .finish()
    }
}

impl<T: Display> Display for Span<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Display::fmt(&self.data, f)
    }
}

impl<T: Hash> Hash for Span<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl<T: PartialEq> PartialEq for Span<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<T: PartialEq> Eq for Span<T> {}

pub trait ToSpan {
    fn span(self, rng: [usize; 2]) -> Span<Self>
    where
        Self: Sized,
    {
        Span { rng, data: self }
    }
}

impl<T> ToSpan for T {}
