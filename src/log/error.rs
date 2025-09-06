use std::fmt::Display;

use crate::log::{Log, Logger, Point};

#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => {{
        eprint!("{}: ", colored::Colorize::bright_red("error"));
        eprintln!($($arg)*);
        std::process::exit(1);
    }};
}

impl Logger {
    pub fn err<'a, S: AsRef<str> + Display + From<&'a str>>(&mut self, msg: S) -> Option<!> {
        self.call(&mut [(self.rng, Point::Error, "")], Log::Error, msg, "");

        None
    }

    pub fn err_op<T: Display>(&mut self, after: bool, op: &[T]) -> Option<!> {
        let mut iter = op.iter().map(|e| format!("`{e}`"));
        let mut msg = "expected ".to_string();

        if op.len() > 2 {
            for _ in 0..op.len() - 2 {
                msg += &(iter.next().unwrap() + ", ");
            }
        }

        msg += &iter.next().unwrap();

        if let Some(s) = iter.next() {
            msg += &format!(" or {s}");
        }

        if after {
            msg += " thereafter"
        }

        self.err(msg)
    }

    pub fn err_mul<'a, S: AsRef<str> + Display + From<&'a str>, const N: usize>(
        &mut self,
        pnt: [[usize; 2]; N],
        msg: S,
    ) -> Option<!> {
        self.call(&mut pnt.map(|v| (v, Point::Error, "")), Log::Error, msg, "");

        None
    }

    pub fn err_rng<'a, S: Display + AsRef<str> + From<&'a str>>(
        &mut self,
        rng: [usize; 2],
        msg: S,
    ) -> Option<!> {
        self.rng = rng;
        self.err(msg)
    }
}
