use std::{fmt::Display, process::exit};

use colored::Colorize;
use unicode_width::UnicodeWidthStr;

use crate::source::Source;

pub fn error(msg: &str) -> ! {
    eprintln!("{}: {msg}", "error".red());
    exit(1)
}

impl Source {
    fn code_line(&self, pnt: &[[usize; 2]]) {
        let mut iter = pnt.into_iter().peekable();
        let pad = self.line.len().to_string().len();
        let border = format!("{} - ", " ".repeat(pad)).black().to_string();
        let mut buf = format!("{border}\n");

        while let Some(mut rng) = iter.next() {
            let mut start = match self.data[..=rng[0]]
                .into_iter()
                .rev()
                .position(|c| *c == '\n')
            {
                Some(n) => rng[0] - n + 1,
                _ => 0,
            };
            let end = match self.data[start..].into_iter().position(|c| *c == '\n') {
                Some(n) => start + n - 1,
                None => self.data.len() - 1,
            };
            let line = match self.line.iter().position(|n| n + 1 == start) {
                Some(n) => n + 2,
                None => 1,
            }
            .to_string();
            let code = self.data[start..=end].into_iter().collect::<String>();

            buf += &format!(
                "{}{code}\n{border}",
                format!("{line}{} | ", " ".repeat(pad - line.len())).black(),
            );

            loop {
                let eof = (rng[0] > rng[1]) as usize;
                let space = code[0..rng[0] + eof - start].width();
                let point = rng[1].checked_sub(rng[0]).unwrap_or_default() + 1;

                buf += &format!("{}{}", " ".repeat(space), "^".repeat(point).red());

                if let Some(tmp) = iter.peek() {
                    if tmp[0] > end {
                        break;
                    }

                    start = rng[1] + 1;
                    rng = tmp;
                    iter.next();
                } else {
                    break;
                }
            }

            buf.push('\n');
        }

        eprint!("{buf}");
    }

    pub fn err(&mut self, msg: &str) -> ! {
        self.code_line(&[match self.rng == [0; 2] {
            true => [self.idx; 2],
            _ => self.rng,
        }]);
        error(msg)
    }

    pub fn err_op<T: Display>(&mut self, mut after: bool, op: &[T]) -> ! {
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

        if let Some(n) = self.de.last() {
            if *n == self.idx {
                after = true
            }
        } else if self.idx > self.rng[1] && !self.data[self.idx].is_ascii_whitespace() {
            self.rng.fill(0)
        }

        if after {
            msg += " thereafter"
        }

        self.err(&msg)
    }

    pub fn err_mul(&mut self, pnt: &mut [[usize; 2]], msg: &str) {
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

        self.code_line(pnt);
        error(msg)
    }

    pub fn err_rng(&mut self, rng: [usize; 2], msg: &str) {
        self.rng = rng;
        self.err(msg);
    }

    pub fn eof(&mut self) -> ! {
        self.code_line(&[[self.idx, 0]]);
        println!();
        error("unexpected end of file")
    }
}
