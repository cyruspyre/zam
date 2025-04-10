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
