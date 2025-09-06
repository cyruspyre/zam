mod error;
mod misc;

use std::{
    borrow::Cow,
    fmt::Display,
    io::{BufWriter, Write, stderr},
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
    pub ctx: Option<([usize; 2], Point, Cow<'static, str>)>,
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

        let tmp = match &self.ctx {
            Some(v) => v.0[0],
            _ => 0,
        };
        let tmp = tmp.max(pnt.last().unwrap().0[0]);

        loop {
            let last = match self.line.last() {
                Some(n) => n + 1,
                _ => 0,
            };

            if last > tmp {
                break;
            }

            let tmp = match self.data[last..].iter().position(|c| *c == '\n') {
                Some(n) => last + n,
                _ => self.data.len(),
            };

            self.line.push(tmp);
        }

        let pad = self
            .line
            .binary_search(&tmp)
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
        let mut iter = pnt.bypass().into_iter().peekable();
        let mut val = None;
        let mut tmp = true;

        io.write(buf.as_bytes()).unwrap();

        loop {
            let Some((rng, pnt, label)) = val else {
                let Some((rng, pnt, label)) = iter.bypass().peek() else {
                    io.write(format!("{border}\n").as_bytes()).unwrap();
                    break;
                };

                val = if tmp
                    && let Some(ctx) = &self.ctx
                    && ctx.0[0] < rng[0]
                {
                    tmp = false;
                    Some((ctx.0, ctx.1, Cow::Borrowed(ctx.2.as_ref())))
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
            let line_idx = self.line.binary_search(&rng[0]).either();
            let line = line_idx.add(1).to_string().bright_black();
            let start = match self.line.get(line_idx.wrapping_sub(1)) {
                Some(n) => n + 1,
                _ => 0,
            };
            let end = self.line.get(line_idx).unwrap() - 1;
            let code: String = self.data[start..=end].iter().collect();
            let blank = format!("\n{}{} ", " ".repeat(pad + line.len()), "|".bright_black());
            let buf = format!(
                "{border}\n{line}{} {} {code}{blank}",
                " ".repeat(pad - line.len()),
                "|".bright_black(),
            );

            io.write(buf.as_bytes()).unwrap();

            // maybe i can rewrite the following parts in a better way...
            let mut labels = vec![(rng, pnt, label)];
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
                labels.push((rng, *pnt, label));
            }

            let stamp = labels.len();

            loop {
                let mut start = start;
                let mut idx = 0;

                while let Some((rng, pnt, label)) = labels.bypass().get_mut(idx) {
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
                        " ".repeat(rng[0] - start),
                        pnt.repeat(rng[1] - rng[0] + 1).color(color)
                    );

                    io.write(buf.as_bytes()).unwrap();

                    start = rng[1] + 1;
                    rng[1] = rng[0];
                    idx += 1;

                    if idx == labels.len() {
                        if idx != stamp {
                            io.write(format!("{blank}{buf}").as_bytes()).unwrap();
                        }

                        let buf = format!(" {}", label.color(color));

                        io.write(buf.as_bytes()).unwrap();
                        labels.pop();
                    }
                }

                if !flag {
                    flag = true;
                    labels.retain(|v| !v.2.is_empty());
                }

                if labels.is_empty() {
                    break;
                }

                io.write(blank.as_bytes()).unwrap();
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
