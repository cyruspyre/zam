use std::{
    borrow::Cow,
    fmt::Display,
    io::{stderr, BufWriter, Write},
    ops::Add,
};

use colored::{Color, Colorize};

use crate::misc::{Bypass, Either};

use super::{Context, Parser};

#[derive(Debug)]
pub enum Log {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy)]
pub enum Point {
    None,
    Info,
    Error,
    Warning,
}

impl Parser {
    pub fn log<L, M, N>(
        &mut self,
        mut pnt: &mut [([usize; 2], Point, L)],
        typ: Log,
        msg: M,
        note: N,
    ) where
        L: Display + AsRef<str>,
        M: Display + AsRef<str>,
        N: Display + AsRef<str>,
    {
        if self.ignore {
            return;
        }

        if let Log::Error = typ {
            self.err += 1
        }

        pnt.sort_unstable_by_key(|v| v.0[0]);

        let mut io = BufWriter::new(stderr().lock());
        let mut iter = pnt.bypass().into_iter().peekable();
        let mut val = if let Some(ctx) = &self.ctx {
            let name = match **ctx {
                Context::Struct => "struct",
                Context::Function => "function",
            };

            Some((
                ctx.rng,
                Point::Info,
                Cow::<str>::Owned(format!("while parsing this {name}")),
            ))
        } else {
            None
        };
        let line = |n: usize| {
            self.line
                .binary_search(&n)
                .either()
                .add(1)
                .ilog10()
                .add(1)
                .try_into()
                .unwrap()
        };
        let pad = line(pnt.last().unwrap().0[0]);
        let border = format!("{} {}", " ".repeat(pad), "-".black());
        let typ = match typ {
            Log::Error => "error".red(),
            Log::Warning => "warning".bright_yellow(),
        };
        let first = pnt[0].0[0];
        let tmp = self.line.binary_search(&first).either();
        let buf = format!(
            "{typ}: {msg}\n{}{} {}:{}:{}\n",
            " ".repeat(pad),
            "-->".black(),
            self.path.display(),
            tmp + 1,
            first - self.line.get(tmp.wrapping_sub(1)).unwrap_or(&0),
        );

        io.write(buf.as_bytes()).unwrap();

        loop {
            let Some((rng, pnt, label)) = val else {
                val = if let Some((rng, pnt, label)) = iter.next() {
                    Some((
                        *rng,
                        *pnt,
                        Cow::Borrowed(unsafe { &*(label.as_ref() as *const _) }),
                    ))
                } else {
                    break;
                };

                continue;
            };
            let tmp = rng[0].max(rng[1]);

            if match self.line.last() {
                Some(n) => *n,
                _ => 0,
            } < tmp
            {
                let tmp = match self.data[tmp..].iter().position(|c| *c == '\n') {
                    Some(n) => tmp + n,
                    _ => self.data.len(),
                };

                self.line.push(tmp);
            }

            let tmp = self.line.binary_search(&rng[0]).either();
            let line = tmp.add(1).to_string().black();
            let start = match self.line.get(tmp.wrapping_sub(1)) {
                Some(n) => n + 1,
                _ => 0,
            };
            let end = self.line.get(tmp).unwrap() - 1;
            let code: String = self.data[start..=end].iter().collect();
            let buf = format!(
                "{border}\n{line}{} {} {code}\n{border} ",
                " ".repeat(pad - line.len()),
                "|".black(),
            );

            io.write(buf.as_bytes()).unwrap();

            let mut labels = vec![(1, rng, pnt, label)];

            while let Some((rng, pnt, label)) = iter.bypass().peek_mut() {
                let (rng, label) = if end > rng[1] {
                    let tmp = Cow::Borrowed(unsafe { &*(label.as_ref() as *const _) });

                    (*rng, tmp)
                } else if end > rng[0] {
                    let tmp = *rng;

                    rng[0] = end + 1;
                    (tmp, Cow::Borrowed(""))
                } else {
                    break;
                };

                iter.next();
                labels.push((labels.len() + 1, rng, *pnt, label));
            }

            while labels.len() != 0 {
                let mut tmp = start;

                for (i, rng, pnt, label) in labels.bypass() {
                    let color = match pnt {
                        Point::Info => Color::BrightBlue,
                        Point::Error => Color::Red,
                        _ => Color::BrightYellow,
                    };
                    let pnt_ = match pnt {
                        _ if rng[0] > rng[1] => "|",
                        Point::Info => "-",
                        Point::None => "",
                        _ => "^",
                    };
                    let buf = format!(
                        "{}{}",
                        " ".repeat(rng[0].min(rng[1]) - tmp),
                        pnt_.repeat(rng[1].saturating_sub(rng[0]) + 1).color(color)
                    );

                    tmp = rng[1] + 1;

                    if rng[1] > rng[0] {
                        rng.swap(0, 1);
                    }

                    io.write(buf.as_bytes()).unwrap();

                    if *i == labels.len() {
                        let buf = format!(" {}", label.color(color));

                        io.write(buf.as_bytes()).unwrap();
                    }
                }

                labels.pop();

                if labels.len() != 0 {
                    let buf = format!("\n{}  ", " ".repeat(pad + line.len()));

                    io.write(buf.as_bytes()).unwrap();
                }
            }

            io.write(b"\n").unwrap();
            val = None
        }

        if !note.as_ref().is_empty() {
            let buf = format!(
                "{} {} {}: {note}\n",
                " ".repeat(pad),
                "=".black(),
                "note".bold()
            );
            io.write(buf.as_bytes()).unwrap();
        }

        io.flush().unwrap();
    }
}
