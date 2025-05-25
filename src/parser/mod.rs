pub mod error;
pub mod log;
pub mod misc;
pub mod span;

use std::{any::TypeId, collections::VecDeque, fmt::Display, path::PathBuf};

use log::{Log, Point};
use misc::{read_file, CharExt};
use span::Span;

use crate::misc::{Either, Result};

pub enum Context {
    Struct,
    Function,
}

#[derive(Default)]
pub struct Parser {
    pub path: PathBuf,
    pub data: Vec<char>,
    pub line: Vec<usize>,
    pub rng: [usize; 2],
    pub idx: usize,
    pub err: usize,
    pub ctx: Option<Span<Context>>,
    pub ignore: bool,
    pub de: VecDeque<usize>,
}

impl Parser {
    pub fn new(path: PathBuf) -> Option<Self> {
        let data = read_file(&path).chars().collect();

        Some(Self {
            data,
            path,
            line: Vec::new(),
            err: 0,
            rng: [0; 2],
            idx: usize::MAX,
            ctx: None,
            ignore: false,
            de: VecDeque::new(),
        })
    }

    #[inline]
    pub fn is_eof(&self) -> bool {
        self.idx == self.data.len() - 1
    }

    pub fn _next<'a>(&'a mut self) -> Option<char> {
        if let Some(c) = self._peek() {
            self.idx = self.idx.wrapping_add(1);

            if c == '\n'
                && match self.line.last() {
                    Some(v) => *v < self.idx,
                    _ => true,
                }
            {
                self.line.push(self.idx);
            }

            return Some(c);
        }

        None
    }

    pub fn next(&mut self) -> char {
        self._next().unwrap()
    }

    // todo: this looks hideous try to rewrite it
    pub fn next_if<T: ToString>(&mut self, op: &[T]) -> Result<String> {
        let tmp = self.idx;
        let mut rng = self.rng;
        let mut op: Vec<_> = op.into_iter().map(|v| v.to_string()).collect();
        let mut buf = String::new();
        let mut ok = false;
        let de = *self.de.back().unwrap_or(&0);
        let mut early = true;

        op.sort_unstable();

        while let Some(c) = self._next() {
            if c.is_ascii_whitespace() {
                if buf.len() != 0 {
                    break;
                }

                continue;
            }

            if buf.is_empty() {
                rng.fill(self.idx);
            }

            buf.push(c);

            if op.binary_search(&buf).is_ok() {
                ok = true;
                break;
            }

            if de == self.idx {
                self.idx -= 1;
                early = false;
                buf.pop();
                break;
            }
        }

        if !ok {
            self.idx = tmp;
        }

        if ok || early {
            rng[1] += buf.len().checked_sub(1).unwrap_or_default();
            self.rng = rng;
        }

        match ok {
            true => Ok(buf),
            _ => Err(buf),
        }
    }

    pub fn next_char_if(&mut self, op: &[char]) -> char {
        self.next_if(op).either().chars().next().unwrap_or_default()
    }

    pub fn _peek(&mut self) -> Option<char> {
        if let Some(c) = self.data.get(self.idx.wrapping_add(1)) {
            if *c == '/'
                && !matches!(self.data[self.rng[0]], '"' | '\'')
                && self
                    .data
                    .get(self.idx.wrapping_add(2))
                    .is_some_and(|c| *c == '/')
            {
                for c in &self.data[self.idx.wrapping_add(1)..] {
                    if *c == '\n' {
                        return self._peek();
                    }

                    self.idx = self.idx.wrapping_add(1);
                }
            }

            return Some(*c);
        }

        None
    }

    pub fn peek(&mut self) -> char {
        if let Some(c) = self._peek() {
            return c;
        }

        '\0'
    }

    pub fn peek_more(&mut self) -> char {
        if let Some(c) = self.data.get(self.idx.wrapping_add(2)) {
            return *c;
        }

        '\0'
    }

    pub fn word(&mut self) -> String {
        let mut buf = String::new();

        while let Some(c) = self._peek() {
            if buf.is_empty() && c.is_ascii_whitespace() {
                self.next();
                continue;
            }

            if !c.is_id() {
                break;
            }

            if buf.is_empty() {
                self.rng[0] = self.idx + 1;
            }

            buf.push(self.next());
        }

        if buf.len() != 0 {
            self.rng[1] = self.rng[0] + buf.len() - 1
        }

        buf
    }

    // fn _identifier(&mut self, required: bool) -> Option<Identifier> {dbg!(self.rng);
    //     let tmp = self.word();
    //     dbg!(self.rng);
    //     let mut err = |msg: &str| {
    //         self.log(
    //             &mut [(self.rng, Point::Error, msg)],
    //             Log::Error,
    //             format!("expected `<identifier>`",),
    //             "",
    //         );

    //         None::<!>
    //     };

    //     if required && tmp.is_empty() {
    //         let rng = self.rng;
    //         let mut after = self.id_or_symbol().is_none();

    //         if self.data[self.rng[0] - 1] == '\n' {
    //             self.rng = rng;
    //             after = true;
    //         }

    //         self.err_op(after, &["<identifier>"])?
    //     }

    //     if tmp.chars().next().is_some_and(|c| c.is_ascii_digit()) {
    //         err("cannot start with a number")?
    //     }

    //     if matches!(
    //         tmp.as_str(),
    //         "fn" | "if"
    //             | "in"
    //             | "cte"
    //             | "let"
    //             | "pub"
    //             | "use"
    //             | "for"
    //             | "else"
    //             | "loop"
    //             | "enum"
    //             | "true"
    //             | "false"
    //             | "while"
    //             | "struct"
    //             | "extern"
    //     ) {
    //         err("cannot be a keyword")?
    //     }

    //     Some(tmp.span(self.rng))
    // }

    // pub fn identifier(&mut self, required: bool) -> Option<Identifier> {
    //     let idx = self.idx;
    //     let tmp = self._identifier(required);

    //     if self.ignore && tmp.is_none() {
    //         self.idx = idx
    //     }

    //     tmp
    // }

    pub fn skip_whitespace(&mut self) -> char {
        while let Some(c) = self._peek() {
            if c.is_ascii_whitespace() {
                self.next();
                continue;
            }

            return c;
        }

        '\0'
    }

    pub fn enclosed(&mut self, de: char) -> Option<String> {
        self.expect_char(&[de])?;
        self.rng = [self.idx; 2];

        let mut buf = String::new();

        while let Some(c) = self._next() {
            if c.is_ascii_whitespace() {
                continue;
            }

            if c == de {
                self.rng[1] = self.idx;

                if buf.is_empty() {
                    self.err("expected a value inside it")?
                }

                return Some(buf);
            }

            buf.push(c);
        }

        self.err(format!("unclosed delimiter `{de}` starting here"))?
    }

    #[must_use]
    pub fn ensure_closed(&mut self, de: char) -> Option<()> {
        let tmp = self.idx;
        let typ = self.data[tmp];
        let same = typ == de;
        let mut count = 0;
        let mut string = 0;

        'a: while let Some(c) = self._next() {
            if c.is_quote() && !same {
                string = self.idx;

                while let Some(v) = self._next() {
                    if c == v && self.data[self.idx - 1] != '\\' {
                        string = 0;
                        continue 'a;
                    }
                }

                break;
            }

            if c == de {
                if count == 0 {
                    self.de.push_back(self.idx);
                    self.idx = tmp;
                    return Some(());
                } else {
                    count -= 1
                }
            } else if c == typ {
                count += 1
            }
        }

        let mut pnt = Vec::with_capacity(3);
        let label = match string {
            0 => String::new(),
            _ => format!(
                "unclosed {} literal",
                match self.data[string] {
                    '"' => "string",
                    _ => "character",
                }
            ),
        };

        pnt.push(([tmp; 2], Point::Info, "starting here"));

        if string != 0 {
            pnt.push(([string; 2], Point::Info, &label))
        }

        pnt.push(([self.idx + 1; 2], Point::Error, ""));

        self.log(
            &mut pnt,
            Log::Error,
            format!("unclosed delimeter `{de}`"),
            "",
        );

        None
    }

    pub fn id_or_symbol(&mut self) -> Option<String> {
        self.skip_whitespace();

        let tmp = self._next()?;
        let typ = tmp.is_id();
        let mut buf = tmp.to_string();

        self.rng.fill(self.idx);

        while let Some(c) = self._next() {
            if c.is_ascii_whitespace() || c.is_id() != typ {
                break;
            }

            buf.push(c);
        }

        self.idx -= 1;
        self.rng[1] = self.idx;

        Some(buf)
    }

    pub fn might<T: Display>(&mut self, t: T) -> bool {
        let rng = self.rng;
        let ok = self.next_if(&[t]).is_ok();

        if !ok {
            self.rng = rng
        }

        ok
    }

    #[must_use]
    pub fn expect<T: Display + 'static>(&mut self, op: &[T]) -> Option<String> {
        let rng = self.rng;
        let tmp = self.next_if(op);

        if let Err(tmp) = tmp {
            let idx = self.rng[0];
            let multiline = self
                .line
                .get(self.line.binary_search(&rng[0]).either())
                .is_some_and(|v| idx > *v);

            if !tmp.is_empty() && TypeId::of::<T>() == TypeId::of::<char>() {
                self.rng[0] = self.rng[1];
            }

            if multiline {
                self.rng = rng
            }

            self.err_op(tmp.is_empty() || multiline, &op)?
        }

        tmp.ok()
    }

    #[must_use]
    pub fn expect_char(&mut self, op: &[char]) -> Option<char> {
        self.expect(op)?;
        Some(self.data[self.idx])
    }
}
