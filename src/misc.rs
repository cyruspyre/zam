use std::{
    borrow::Borrow,
    fmt::{Debug, Formatter},
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    ptr::{null, null_mut},
};

use colored::Colorize;

pub type Result<T> = std::result::Result<T, T>;

/// A trait for bypassing Rust's lifetime rules in specific scenarios.
pub trait Bypass {
    /// Bypasses rust's lifetime rules
    ///
    /// # Warning
    /// The caller must ensure it's okay to have multiple mutable references otherwise it's undefined behavior.
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

/// A wrapper around `Drop` implementation to perform custom cleanup in niche cases.
///
/// It is useful when you have to perform cleanup when breaking/returning early or come across an error while error propagating.
///
/// # Examples
///
/// Correct usage:
/// ```
/// let __ = CustomDrop(|| {}); // Drops when it goes out of scope
/// ```
///
/// *Same goes for any function returning `CustomDrop`*
///
/// Incorrect usage:
/// ```
/// CustomDrop(|| {}); // Drops right away
/// let _ = CustomDrop(|| {}); // also drops right away
/// ```
pub struct CustomDrop<F: FnMut()>(pub F);

impl<F: FnMut()> Drop for CustomDrop<F> {
    #[inline]
    fn drop(&mut self) {
        self.0()
    }
}

pub struct Ref<T>(pub *const T);

impl<T> Borrow<T> for Ref<T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<T> Borrow<T> for &Ref<T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<T> Deref for Ref<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl<T: Hash> Hash for Ref<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}

impl<T: PartialEq> PartialEq for Ref<T> {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}

impl<T: PartialEq> Eq for Ref<T> {}

impl<T> Default for Ref<T> {
    fn default() -> Self {
        Self(null())
    }
}

impl<T> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T> Copy for Ref<T> {}

impl<T: Debug> Debug for Ref<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe { (*self.0).fmt(f) }
    }
}

#[derive(PartialEq)]
pub struct RefMut<T>(pub *mut T);

impl<T> RefMut<T> {
    #[inline]
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }
}

impl<T> Default for RefMut<T> {
    fn default() -> Self {
        Self(null_mut())
    }
}

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

impl<T> Clone for RefMut<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T> Debug for RefMut<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            return match self.is_null() {
                true => f.write_str(&"null".black().italic().to_string()),
                _ => self.fmt(f),
            };
        }
        f.debug_tuple("RefMut").field(&self.0).finish()
    }
}
