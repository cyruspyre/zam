use std::{
    fmt::Display,
    io::{stderr, BufWriter, Write},
    ops::{Add, Sub},
};

use colored::{Color, Colorize};

use super::Parser;

#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => {{
        // eprint!("{}: ", colored::Colorize::red("error"));
        // eprintln!($($arg)*);
        std::process::exit(1);
    }};
}

pub enum LogType {
    Error,
    Warning,
}

impl Parser {
    pub fn log<S: Display + AsRef<str>>(&mut self, pnt: &[([usize; 2], S)], typ: LogType, msg: S) {
        let color = match typ {
            LogType::Error => Color::Red,
            _ => Color::BrightYellow,
        };
        let last_line = self
            .line
            .binary_search(&pnt.last().unwrap().0[0])
            .unwrap_err()
            .add(1)
            .to_string()
            .len();
        let pad = " ".repeat(last_line + 1);
        let border = format!("{pad}- ").black();
        let mut iter = pnt.into_iter().peekable();
        let mut io = BufWriter::new(stderr().lock());
        let mut tmp = true;

        while let Some((mut rng, label)) = iter.next() {
            let idx = self.line.binary_search(&rng[0]).unwrap_err();
            let mut start = match self.line.get(idx.wrapping_sub(1)) {
                Some(v) => v + 1,
                _ => 0,
            };
            let end = match self.line.get(idx) {
                Some(v) => *v,
                _ => self.data.len(),
            } - 1;
            let line = (idx + 1).to_string().black();
            let code: String = self.data[start..=end].into_iter().collect();

            if tmp {
                io.write(
                    format!(
                        "{}{} {}:{}:{}\n{border}\n",
                        " ".repeat(last_line),
                        "-->".black(),
                        self.path.display(),
                        idx + 1,
                        rng[0] - start + 1
                    )
                    .as_bytes(),
                )
                .unwrap();
                tmp = false;
            }

            io.write(
                format!(
                    "{line}{} {} {code}\n{border}",
                    " ".repeat(last_line - line.len()),
                    "|".black()
                )
                .as_bytes(),
            )
            .unwrap();

            let mut label = label;
            let mut lebels = Vec::with_capacity(pnt.len());

            loop {
                io.write(
                    format!(
                        "{}{}",
                        " ".repeat(rng[0] - start),
                        "^".repeat(rng[1] - rng[0] + 1)
                    )
                    .color(color)
                    .to_string()
                    .as_bytes(),
                )
                .unwrap();

                match iter.next_if(|v| v.0[1] < end) {
                    Some((rng_, label_)) => {
                        if !label.as_ref().is_empty() {
                            lebels.push((
                                lebels
                                    .last()
                                    .map(|v: &(usize, _)| v.0)
                                    .unwrap_or_default()
                                    .add(rng[0])
                                    .sub(start),
                                label,
                            ));
                        }

                        start = rng[1] + 1;
                        label = label_;
                        rng = *rng_;
                    }
                    _ => {
                        io.write(format!(" {label}\n").color(color).to_string().as_bytes())
                            .unwrap();
                        break;
                    }
                };
            }

            let tmp = unsafe { &mut *(&mut lebels as *mut Vec<_>) };

            while lebels.len() != 0 {
                io.write(" ".repeat(border.len()).as_bytes()).unwrap();

                for (i, (pad, label)) in lebels.iter().enumerate() {
                    io.write(
                        format!("{}| ", " ".repeat(*pad))
                            .color(color)
                            .to_string()
                            .as_bytes(),
                    )
                    .unwrap();

                    if lebels.len() - i == 1 {
                        io.write(label.as_ref().color(color).to_string().as_bytes())
                            .unwrap();
                        tmp.pop();
                    }
                }

                io.write(b"\n").unwrap();
            }

            if iter.peek().is_some() {
                io.write(format!("{border}\n").as_bytes()).unwrap();
            }
        }

        io.write(
            format!(
                "{}: {msg}\n",
                match typ {
                    LogType::Error => "error",
                    _ => "warning",
                }
                .color(color)
            )
            .as_bytes(),
        )
        .unwrap();

        io.flush().unwrap()
    }

    pub fn err(&mut self, msg: &str) -> ! {
        self.log(
            &mut [(
                match self.rng == [0; 2] {
                    true => [self.idx; 2],
                    _ => self.rng,
                },
                "",
            )],
            LogType::Error,
            msg,
        );
        err!("{msg}")
    }

    pub fn err_op<T: Display>(&mut self, after: bool, op: &[T]) -> ! {
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

        self.err(&msg)
    }

    pub fn err_mul<'a, S: AsRef<str> + Display + From<&'a str>>(
        &mut self,
        pnt: &mut [[usize; 2]],
        msg: S,
    ) {
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
                .map(|v| (*v, "".into()))
                .collect::<Vec<_>>()
                .as_slice(),
            LogType::Error,
            msg,
        );
        err!("{msg}")
    }

    pub fn err_rng(&mut self, rng: [usize; 2], msg: &str) -> ! {
        self.rng = rng;
        self.err(msg);
    }

    pub fn eof(&mut self) -> ! {
        self.log(&[([self.idx, 0], "")], LogType::Error, "unexpected end of file");
        err!("\nunexpected end of file")
    }
}
