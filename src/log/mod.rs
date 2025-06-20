mod error;

use std::{
    borrow::Cow,
    fmt::Display,
    io::{stderr, BufWriter, Write},
    ops::Add,
    path::PathBuf,
};

use colored::{Color, Colorize};

use crate::misc::{Bypass, Either};

#[derive(Default)]
pub struct Logger {
    pub path: PathBuf,
    pub data: Vec<char>,
    pub line: Vec<usize>,
    pub rng: [usize; 2],
    pub eof: bool,
    pub err: usize,
    pub ctx: Option<([usize; 2], Point, &'static str)>,
    pub ignore: bool,
}

#[derive(Debug)]
pub enum Log {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Point {
    None,
    Info,
    Error,
    Warning,
}

impl Logger {
    pub fn call<L, M, N>(
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

        let mut io = BufWriter::new(stderr().lock());
        let typ = match typ {
            Log::Error => "error".bright_red(),
            Log::Warning => "warning".bright_yellow(),
        };

        io.write(format!("{typ}: {msg}\n").as_bytes()).unwrap();

        if pnt.is_empty() {
            return io.flush().unwrap();
        }

        pnt.sort_unstable_by_key(|v| v.0[0]);

        let mut iter = pnt.bypass().into_iter().peekable();
        let mut val = None;
        let pad = self
            .line
            .binary_search(&pnt.last().unwrap().0[0])
            .either()
            .add(1)
            .ilog10()
            .add(1)
            .try_into()
            .unwrap();
        let border = format!("{} {}", " ".repeat(pad), "-".bright_black());
        let first = pnt[0].0[0];
        let tmp = self.line.binary_search(&first).either();
        let buf = format!(
            "{}{} {}:{}:{}\n",
            " ".repeat(pad),
            "-->".bright_black(),
            self.path.display(),
            tmp + 1,
            first - self.line.get(tmp.wrapping_sub(1)).unwrap_or(&0),
        );
        let mut tmp = true;

        io.write(buf.as_bytes()).unwrap();

        loop {
            let Some((rng, pnt, label)) = val else {
                let Some((rng, pnt, label)) = iter.bypass().peek() else {
                    break;
                };

                val = if tmp
                    && let Some(ctx) = &self.ctx
                    && ctx.0[0] < rng[0]
                {
                    tmp = false;
                    Some((ctx.0, ctx.1, Cow::Borrowed(ctx.2)))
                } else {
                    iter.next();
                    Some((
                        *rng,
                        *pnt,
                        Cow::Borrowed(unsafe { &*(label.as_ref() as *const _) }),
                    ))
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

            let line_idx = self.line.binary_search(&rng[0]).either();
            let line = line_idx.add(1).to_string().bright_black();
            let start = match self.line.get(line_idx.wrapping_sub(1)) {
                Some(n) => n + 1,
                _ => 0,
            };
            let end = self.line.get(line_idx).unwrap() - 1;
            let code: String = self.data[start..=end].iter().collect();
            let buf = format!(
                "{border}\n{line}{} {} {code}\n{border} ",
                " ".repeat(pad - line.len()),
                "|".bright_black(),
            );

            io.write(buf.as_bytes()).unwrap();

            let mut labels = vec![(1, rng, pnt, label)];
            let mut flag = false;

            while let Some((rng, pnt, label)) = iter.bypass().peek_mut() {
                let (rng, label) = if end >= rng[1] {
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
                        Point::Error => Color::BrightRed,
                        _ => Color::BrightYellow,
                    };
                    let pnt = match pnt {
                        _ if flag => "|",
                        Point::Info => "-",
                        Point::None => "",
                        _ => "^",
                    };
                    let buf = format!(
                        "{}{}",
                        " ".repeat(rng[0].min(rng[1]) - tmp),
                        pnt.repeat(rng[1].saturating_sub(rng[0]) + 1).color(color)
                    );

                    tmp = rng[1] + *i;

                    if rng[1] > rng[0] {
                        rng.swap(0, 1);
                    }

                    io.write(buf.as_bytes()).unwrap();

                    if *i == labels.len() {
                        let buf = format!(" {}", label.color(color));

                        io.write(buf.as_bytes()).unwrap();
                    }
                }

                flag = true;
                labels.pop();

                if labels.len() != 0 {
                    let buf = format!("\n{}  ", " ".repeat(pad + line.len()));

                    io.write(buf.as_bytes()).unwrap();
                }
            }

            io.write(b"\n").unwrap();
            val = None;
        }

        if !note.as_ref().is_empty() {
            let buf = format!(
                "{} {} {}: {note}\n",
                " ".repeat(pad),
                "=".bright_black(),
                "note".bold()
            );
            io.write(buf.as_bytes()).unwrap();
        }

        io.flush().unwrap();
    }
}

type Args<'a, P, M, N> = (&'a mut [([usize; 2], Point, P)], Log, M, N);

impl<P, M, N> FnOnce<Args<'_, P, M, N>> for Logger
where
    P: Display + AsRef<str>,
    M: Display + AsRef<str>,
    N: Display + AsRef<str>,
{
    type Output = ();

    extern "rust-call" fn call_once(mut self, args: Args<P, M, N>) -> Self::Output {
        self.call_mut(args)
    }
}

impl<P, M, N> FnMut<Args<'_, P, M, N>> for Logger
where
    P: Display + AsRef<str>,
    M: Display + AsRef<str>,
    N: Display + AsRef<str>,
{
    extern "rust-call" fn call_mut(&mut self, args: Args<P, M, N>) -> Self::Output {
        self.call(args.0, args.1, args.2, args.3);
    }
}
