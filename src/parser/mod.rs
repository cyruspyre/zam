pub mod misc;
pub mod span;

use std::{any::type_name, collections::VecDeque, fmt::Display};

use misc::CharExt;

use crate::{
    log::{Log, Logger, Point},
    misc::{Bypass, Either, Ref, RefMut, Result},
    zam::{block::Impls, path::ZamPath},
};

#[derive(Default)]
pub struct Parser {
    pub id: Ref<ZamPath>,
    pub log: Logger,
    pub idx: usize,
    pub de: VecDeque<usize>,
    pub impls: RefMut<Impls>,
}

impl Parser {
    pub fn de_rng(&mut self) {
        if let Some(v) = self.de.front() {
            self.log.rng[1] = self.log.rng[1].min(v - 1)
        }
    }

    pub fn _next<'a>(&'a mut self) -> Option<char> {
        let line = self.log.line.bypass();

        if let Some(c) = self._peek() {
            self.idx = self.idx.wrapping_add(1);

            if c == '\n'
                && match line.last() {
                    Some(v) => *v < self.idx,
                    _ => true,
                }
            {
                line.push(self.idx);
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
        let mut rng = self.log.rng;
        let mut op: Vec<_> = op.into_iter().map(|v| v.to_string()).collect();
        let mut buf = String::new();
        let mut ok = false;

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
        }

        if !ok {
            self.idx = tmp;
        }

        rng[1] += buf.len().checked_sub(1).unwrap_or_default();
        self.log.rng = rng;

        match ok {
            true => Ok(buf),
            _ => Err(buf),
        }
    }

    pub fn next_char_if(&mut self, op: &[char]) -> char {
        self.next_if(op).either().chars().next().unwrap_or_default()
    }

    pub fn _peek(&mut self) -> Option<char> {
        let Logger { data, rng, .. } = &self.log;

        if let Some(c) = data.get(self.idx.wrapping_add(1)) {
            if *c == '/'
                && !matches!(data[rng[0]], '"' | '\'')
                && data
                    .get(self.idx.wrapping_add(2))
                    .is_some_and(|c| *c == '/')
            {
                for c in &data[self.idx.wrapping_add(1)..] {
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
        if let Some(c) = self.log.data.get(self.idx.wrapping_add(2)) {
            return *c;
        }

        '\0'
    }

    pub fn word(&mut self) -> String {
        let mut buf = String::new();
        let rng = self.log.rng.bypass();

        while let Some(c) = self._peek() {
            if buf.is_empty() && c.is_ascii_whitespace() {
                self.next();
                continue;
            }

            if !c.is_id() {
                break;
            }

            if buf.is_empty() {
                rng[0] = self.idx + 1;
            }

            buf.push(self.next());
        }

        if buf.len() != 0 {
            rng[1] = rng[0] + buf.len() - 1
        }

        buf
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

        let log = self.log.bypass();
        log.rng = [self.idx; 2];
        let mut buf = String::new();

        while let Some(c) = self._next() {
            if c.is_ascii_whitespace() {
                continue;
            }

            if c == de {
                log.rng[1] = self.idx;

                if buf.is_empty() {
                    log.err("expected a value inside it")?
                }

                return Some(buf);
            }

            buf.push(c);
        }

        log.err(format!("unclosed delimiter `{de}` starting here"))?
    }

    #[must_use]
    pub fn ensure_closed(&mut self, de: char) -> Option<usize> {
        let log = self.log.bypass();
        let data = &log.data;
        let tmp = self.idx;
        let typ = data[tmp];
        let same = typ == de;
        let mut count = 0;
        let mut string = 0;

        'a: while let Some(c) = self._next() {
            if c.is_quote() && !same {
                string = self.idx;

                while let Some(v) = self._next() {
                    if c == v && data[self.idx - 1] != '\\' {
                        string = 0;
                        continue 'a;
                    }
                }

                break;
            }

            if c == de {
                if count == 0 {
                    let res = self.idx;

                    self.de.push_front(res);
                    self.idx = tmp;

                    return Some(res);
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
                match data[string] {
                    '"' => "string",
                    _ => "character",
                }
            ),
        };

        pnt.push(([tmp; 2], Point::Info, "starting here"));

        if string != 0 {
            pnt.push(([string; 2], Point::Info, &label))
        }

        pnt.push(([self.idx + 1; 2], Point::Error, "end of file"));

        log(
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
        let rng = self.log.rng.bypass();
        let mut buf = tmp.to_string();

        rng.fill(self.idx);

        while let Some(c) = self._next() {
            if c.is_ascii_whitespace() || c.is_id() != typ {
                self.idx -= 1;
                break;
            }

            buf.push(c);
        }

        rng[1] = self.idx;

        Some(buf)
    }

    pub fn might<T: Display>(&mut self, t: T) -> bool {
        let rng = self.log.rng;
        let ok = self.next_if(&[t]).is_ok();

        if !ok {
            self.log.rng = rng
        }

        ok
    }

    #[must_use]
    pub fn expect<T: Display>(&mut self, op: &[T]) -> Option<String> {
        let rng = self.log.rng;
        let log = self.log.bypass();
        let tmp = self.next_if(op);
        let line = log.line.bypass();

        if let Err(tmp) = tmp {
            let idx = log.rng[0];
            let multiline = line
                .get(line.binary_search(&rng[0]).either())
                .is_some_and(|v| idx > *v);

            if multiline {
                log.rng = rng
            }

            self.de_rng();
            log.err_op(tmp.is_empty() || multiline, &op)?
        }

        tmp.ok()
    }

    #[must_use]
    pub fn expect_char(&mut self, op: &[char]) -> Option<char> {
        self.expect(op)?;
        Some(self.log.data[self.idx])
    }
}
