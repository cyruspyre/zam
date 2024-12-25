use std::fs::read_to_string;

use crate::{block::Block, error::error, misc::ValidID};

pub struct Source {
    pub path: String,
    pub data: Vec<char>,
    pub line: Vec<usize>,
    pub word: String,
    pub rng: [usize; 2],
    pub idx: usize,
    pub de: Vec<usize>,
}

impl Source {
    pub fn new(path: &str) -> Self {
        let data = match read_to_string(path) {
            Ok(v) => v,
            Err(e) => error(&format!("{} [{path}]", e.to_string().to_lowercase())),
        };

        Self {
            path: path.into(),
            data: data.chars().collect(),
            line: Vec::new(),
            word: String::new(),
            rng: [0; 2],
            idx: usize::MAX,
            de: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Block {
        self.block(true)
    }

    pub fn _next<'a>(&'a mut self) -> Option<char> {
        if let Some(c) = self._peek() {
            self.idx = self.idx.wrapping_add(1);

            if c == '\n' {
                self.line.push(self.idx);
            }

            return Some(c);
        }

        None
    }

    pub fn next(&mut self) -> char {
        if let Some(c) = self._next() {
            return c;
        }

        self.eof()
    }

    pub fn next_if(&mut self, op: &[char]) -> char {
        while let Some(c) = self._peek() {
            if c.is_ascii_whitespace() {
                self.next();
                continue;
            }

            for t in op {
                if c == *t {
                    self.next();
                    return c;
                }
            }

            break;
        }

        '\0'
    }

    pub fn _peek(&mut self) -> Option<char> {
        if let Some(c) = self.data.get(self.idx.wrapping_add(1)) {
            if *c == '/'
                && self
                    .data
                    .get(self.idx.wrapping_add(2))
                    .is_some_and(|c| *c == '/')
            {
                for c in &self.data[self.idx.wrapping_add(1)..] {
                    self.idx = self.idx.wrapping_add(1);

                    if *c == '\n' {
                        return self._peek();
                    }
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
            self.rng[1] = self.rng[0] + buf.len() - 1;

            if buf.chars().next().unwrap().is_numeric() {
                self.err("identifiers cannot start with number")
            }
        }

        buf
    }

    pub fn identifier(&mut self) -> String {
        let tmp = self.word();

        if tmp.is_empty() {
            self.skip_whitespace();
            let after = self._next().is_none();
            self.err_op(after, &["<identifier>"]);
        }

        if matches!(
            tmp.as_str(),
            "fn" | "cte" | "let" | "pub" | "enum" | "struct" | "extern"
        ) {
            self.err("identifier cannot be a keyword")
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

    pub fn enclosed(&mut self, de: char) -> String {
        self.expect_char(&[de]);
        self.rng = [self.idx; 2];

        let mut buf = String::new();

        while let Some(c) = self._next() {
            if c.is_ascii_whitespace() {
                continue;
            }

            if c == de {
                self.rng[1] = self.idx;

                if buf.is_empty() {
                    self.err("expected a value inside it")
                }

                return buf;
            }

            buf.push(c);
        }

        self.err(&format!("unclosed delimiter `{de}` starting here"));
    }

    pub fn ensure_closed(&mut self, de: char) {
        let tmp = self.idx;
        let typ = self.data[tmp];
        let mut count = 0;

        while let Some(c) = self._next() {
            if c == typ {
                count += 1
            } else if c == de {
                if count == 0 {
                    self.de.push(self.idx);
                    self.idx = tmp;
                    return;
                } else {
                    count -= 1
                }
            }
        }

        self.err_mul(&mut [[tmp; 2], [self.idx, 0]], "unclosed delimeter")
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

    pub fn might(&mut self, t: char) -> bool {
        while let Some(c) = self._peek() {
            if c.is_ascii_whitespace() {
                self.next();
                continue;
            }

            if c == t {
                self.next();
                self.rng = [self.idx; 2];
                return true;
            }

            break;
        }

        false
    }

    pub fn expect<T: ToString>(&mut self, op: &[T]) -> String {
        let mut buf = String::new();
        let op = op.into_iter().map(|v| v.to_string()).collect::<Vec<_>>();
        let de = match self.de.last() {
            Some(n) => *n,
            _ => 0,
        };

        while let Some(c) = self._next() {
            if self.idx == de {
                break;
            }

            if c.is_ascii_whitespace() {
                if buf.len() != 0 {
                    break;
                }

                continue;
            }

            if buf.is_empty() {
                self.rng = [self.idx; 2];
            } else {
                self.rng[1] += 1;
            }

            buf.push(c);

            if op.binary_search(&buf).is_ok() {
                return buf;
            }
        }

        self.err_op(buf.is_empty(), &op)
    }

    pub fn expect_char(&mut self, op: &[char]) -> char {
        self.expect(op);
        self.data[self.idx]
    }
}
