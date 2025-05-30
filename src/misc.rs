use std::{
    borrow::Borrow,
    fmt::Debug,
    ops::{Deref, DerefMut},
    ptr::null,
};

pub type Result<T> = std::result::Result<T, T>;

/// A trait for bypassing Rust's lifetime rules in specific scenarios.
pub trait Bypass {
    /// Bypasses rust's lifetime rules
    ///
    /// # Warning
    /// The caller must ensure it's okay to have multiple mutable references otherwise it's UB.
    #[inline]
    fn bypass<'a, 'b>(&'a mut self) -> &'b mut Self {
        unsafe { &mut *(self as *mut _) }
    }
}

impl<T> Bypass for T {}

pub trait Either<T> {
    fn either(self) -> T;
}

impl<T> Either<T> for Result<T> {
    fn either(self) -> T {
        match self {
            Ok(v) => v,
            Err(e) => e,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Ref<T: ?Sized>(pub *const T);

impl<T> Default for Ref<T> {
    fn default() -> Self {
        Self(null())
    }
}

impl<T> Deref for Ref<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl<T> Borrow<T> for Ref<T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<T: Debug + ?Sized> Debug for Ref<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { (*self.0).fmt(f) }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RefMut<T>(pub *mut T);

impl<T> Deref for RefMut<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl<T> DerefMut for RefMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}
