use std::fmt::Display;

use super::{
    log::{Log, Point},
    Parser,
};

#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => {{
        eprint!("{}: ", colored::Colorize::red("error"));
        eprintln!($($arg)*);
        std::process::exit(1);
    }};
}

impl Parser {
    pub fn err<'a, S: AsRef<str> + Display + From<&'a str>>(&mut self, msg: S) -> Option<!> {
        self.log(&mut [(self.rng, Point::Error, "")], Log::Error, msg, "");

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

    pub fn err_mul<'a, S: AsRef<str> + Display + From<&'a str>>(
        &mut self,
        pnt: &mut [[usize; 2]],
        msg: S,
    ) -> Option<!> {
        self.log(
            &mut pnt
                .into_iter()
                .map(|v| (*v, Point::Error, ""))
                .collect::<Vec<_>>(),
            Log::Error,
            msg,
            "",
        );

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
