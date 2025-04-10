use std::{
    fmt::Display,
    io::{stderr, BufWriter, Write},
    ops::{Add, Sub},
};

use colored::{Color, Colorize};

use crate::misc::{Bypass, Either};

use super::Parser;

#[derive(Debug)]
pub enum Log {
    Error,
    Warning,
}

#[derive(Debug)]
pub enum Point {
    None,
    Info,
    Error,
    Warning,
}

impl Parser {
    pub fn log<L: Display + AsRef<str>, M: Display + AsRef<str>, N: Display + AsRef<str>>(
        &mut self,
        pnt: &[([usize; 2], Point, L)],
        typ: Log,
        msg: M,
        note: N,
    ) {
        if self.ignore {
            return;
        }

        let last_line = self
            .line
            .binary_search(&pnt.last().unwrap().0[0])
            .either()
            .add(1)
            .to_string()
            .len();
        let pad = " ".repeat(last_line + 1);
        let border = format!("{pad}- ").black();
        let mut iter = pnt.into_iter().peekable();
        let mut io = BufWriter::new(stderr().lock());
        let mut val = None;
        let mut tmp = true;

        loop {
            let Some((ref mut rng, color, indicator, mut label)) = val else {
                val = match iter.next() {
                    Some((a, b, c)) => Some((
                        *a,
                        match b {
                            Point::Warning => Color::BrightYellow,
                            Point::Info => Color::BrightBlue,
                            _ => Color::Red,
                        },
                        match b {
                            Point::None => "",
                            Point::Info => "-",
                            Point::Error | Point::Warning => "^",
                        },
                        c,
                    )),
                    _ => break,
                };

                continue;
            };
            let idx = rng[0].max(rng[1]);

            if match self.line.last() {
                Some(n) => *n,
                _ => 0,
            } < idx
            {
                let tmp = match self.data[idx..].iter().position(|c| *c == '\n') {
                    Some(n) => idx + n,
                    _ => self.data.len(),
                };

                self.line.push(tmp);
            }

            let idx = self.line.binary_search(&rng[0]).either();
            let mut start = match self.line.get(idx.wrapping_sub(1)) {
                Some(v) => v + 1,
                _ => 0,
            };
            let end = self.line.get(idx).unwrap() - 1;
            let eof = (rng[0] <= rng[1]) as usize;
            let line = (idx + 1).to_string().black();
            let code: String = self.data[start..=end].into_iter().collect();

            // // skips a blank line
            // if start > end {
            //     rng[0] += 1;
            //     continue;
            // }

            if tmp {
                io.write(
                    format!(
                        "{}: {msg}\n{}{} {}:{}:{}\n{border}\n",
                        match typ {
                            Log::Error => {
                                self.err += 1;
                                "error".red()
                            }
                            _ => "warning".bright_yellow(),
                        },
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

            let mut labels = Vec::with_capacity(pnt.len());
            loop {
                io.write(
                    format!(
                        "{}{}",
                        " ".repeat(rng[0] - start + 1 - eof),
                        indicator.repeat(rng[1].min(end) - rng[0] * eof + 1)
                    )
                    .color(color)
                    .to_string()
                    .as_bytes(),
                )
                .unwrap();

                match iter.next_if(|v| v.0[0] == v.0[1] && v.0[1] < end) {
                    Some((ref rng_, _, label_)) => {
                        if !label.as_ref().is_empty() {
                            labels.push((
                                labels
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
                        *rng = *rng_;
                    }
                    _ => {
                        io.write(format!(" {label}\n").color(color).to_string().as_bytes())
                            .unwrap();
                        break;
                    }
                };
            }

            let tmp = labels.bypass();

            while labels.len() != 0 {
                io.write(" ".repeat(border.len()).as_bytes()).unwrap();

                for (i, (pad, label)) in labels.iter().enumerate() {
                    io.write(
                        format!("{}| ", " ".repeat(*pad))
                            .color(color)
                            .to_string()
                            .as_bytes(),
                    )
                    .unwrap();

                    if labels.len() - i == 1 {
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

            if rng[1] > end {
                *rng = [end + 2, rng[1]];
                continue;
            }

            val = None
        }

        if !note.as_ref().is_empty() {
            let buf = format!("{pad}{} {}: {note}\n", "=".black(), "note".bold());
            io.write(buf.as_bytes()).unwrap();
        }

        io.flush().unwrap()
    }
}
