pub mod error;
pub mod log;
pub mod misc;
pub mod span;

use std::{
    any::TypeId,
    collections::VecDeque,
    fmt::{Debug, Display},
    path::PathBuf,
};

use log::{Log, Point};
use misc::{read_file, Context, Either, ValidID};
use span::{Identifier, ToSpan};

#[derive(Debug, Default)]
pub struct Parser {
    pub path: PathBuf,
    pub data: Vec<char>,
    pub line: Vec<usize>,
    pub rng: [usize; 2],
    pub idx: usize,
    pub err: usize,
    /// Read-Only mode without throwing error
    pub ro: bool,
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
            ro: false,
            de: VecDeque::new(),
        })
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

    pub fn next_if<T: ToString>(&mut self, op: &[T]) -> Context<bool, String> {
        let tmp = self.idx;
        let mut op: Vec<_> = op.into_iter().map(|v| v.to_string()).collect();
        let mut buf = String::new();
        let mut ctx = false;

        op.sort_unstable();

        while let Some(c) = self._next() {
            if c.is_ascii_whitespace() {
                if buf.len() != 0 {
                    break;
                }

                continue;
            }

            if buf.is_empty() {
                self.rng.fill(self.idx);
            }

            buf.push(c);

            if op.binary_search(&buf).is_ok() {
                ctx = true;
                break;
            }
        }

        self.rng[1] = self.idx;

        if !ctx {
            self.rng[1] -= 1;
            self.idx = tmp;
        }

        Context { ctx, data: buf }
    }

    pub fn next_char_if(&mut self, op: &[char]) -> char {
        self.next_if(op).data.chars().next().unwrap_or_default()
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

    fn _identifier(&mut self, required: bool) -> Option<Identifier> {
        let tmp = self.word();
        let rng = self.rng;

        if required && tmp.is_empty() {
            let after = self.until_whitespace().is_empty();

            if !after && self.de.binary_search(&self.idx).is_err() {
                self.rng = rng
            }

            self.err_op(after, &["<identifier>"])?
        }

        if tmp.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            self.err("identifiers cannot start with number")?
        }

        if matches!(
            tmp.as_str(),
            "fn" | "if"
                | "in"
                | "cte"
                | "let"
                | "pub"
                | "use"
                | "for"
                | "else"
                | "loop"
                | "enum"
                | "while"
                | "struct"
                | "extern"
        ) {
            self.err("identifier cannot be a keyword")?
        }

        Some(tmp.span(rng))
    }

    pub fn identifier(&mut self, required: bool) -> Option<Identifier> {
        let idx = self.idx;
        let tmp = self._identifier(required);

        if self.ro && tmp.is_none() {
            self.idx = idx
        }

        tmp
    }

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
        let mut count = 0;
        let mut string = 0;

        'one: while let Some(c) = self._next() {
            if string != 0 {
                continue;
            }

            if matches!(c, '\'' | '"') {
                string = self.idx;

                while let Some(v) = self._next() {
                    if c == v && self.data[self.idx - 1] != '\\' {
                        string = 0;
                        break;
                    }

                    if v == '{' && !self.might('{') {
                        self.ensure_closed('}')?;

                        if let Some(v) = self.de.back() {
                            self.idx = *v
                        }
                    } else if de == v && !self.might('}') {
                        continue 'one;
                    }
                }
            }

            if c == typ {
                count += 1
            } else if c == de {
                if count == 0 {
                    self.de.push_back(self.idx);
                    self.rng.fill(self.idx);
                    self.idx = tmp;
                    return Some(());
                } else {
                    count -= 1
                }
            }
        }

        let mut pnt = Vec::with_capacity(3);

        pnt.push(([tmp; 2], Point::Info, "starting here"));

        if string != 0 {
            pnt.push(([string; 2], Point::Error, ""))
        }

        pnt.push(([self.idx, 0], Point::Error, ""));

        self.log(
            &pnt,
            Log::Error,
            &format!("unclosed delimeter. expected `{de}`"),
        );

        None
    }

    pub fn until_whitespace(&mut self) -> String {
        let mut buf = String::new();

        while let Some(c) = self._next() {
            if c.is_ascii_whitespace() {
                if buf.is_empty() {
                    continue;
                }

                break;
            }

            if buf.is_empty() {
                self.rng = [self.idx; 2];
            } else {
                self.rng[1] += 1;
            }

            buf.push(c);
        }

        buf
    }

    pub fn might<T: Display>(&mut self, t: T) -> bool {
        let rng = self.rng;
        let tmp = self.next_if(&[t]).ctx;

        if !tmp {
            self.rng = rng
        }

        tmp
    }

    #[must_use]
    pub fn expect<T: Display + 'static>(&mut self, op: &[T]) -> Option<String> {
        let rng = self.rng;
        let tmp = self.next_if(op);

        if !tmp.ctx {
            let idx = self.rng[0];
            let multiline = self
                .line
                .get(self.line.binary_search(&rng[0]).either())
                .is_some_and(|v| idx > *v);

            if TypeId::of::<T>() == TypeId::of::<char>() {
                self.rng.fill(idx)
            }

            if multiline {
                self.rng = rng
            }

            self.err_op(tmp.is_empty() || multiline, &op)?
        }

        Some(tmp.data)
    }

    #[must_use]
    pub fn expect_char(&mut self, op: &[char]) -> Option<char> {
        self.expect(op)?;
        Some(self.data[self.idx])
    }
}
