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
        self.log(
            &mut [(
                match self.rng == [0; 2] {
                    true => [self.idx; 2],
                    _ => self.rng,
                },
                Point::Error,
                "".into(),
            )],
            Log::Error,
            msg,
        );

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
        for n in &self.de {
            if let Ok(i) = pnt.binary_search_by_key(n, |rng| rng[0]) {
                let n = n - self.data[..*n]
                    .iter()
                    .rev()
                    .position(|c| !c.is_ascii_whitespace())
                    .unwrap_or_default();
                pnt[i][0] = n - 1;
            }
        }

        self.log(
            pnt.into_iter()
                .map(|v| (*v, Point::Error, "".into()))
                .collect::<Vec<_>>()
                .as_slice(),
            Log::Error,
            msg,
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
