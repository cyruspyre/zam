use indexmap::IndexMap;

use crate::{
    misc::Bypass,
    parser::span::Span,
    zam::{
        expression::term::Term,
        identifier::Identifier,
        typ::{Type, kind::TypeKind},
    },
};

use super::Parser;

pub type Generic = IndexMap<Identifier, Vec<Trait>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Trait {
    name: Identifier,
    sub: Vec<Trait>,
}

impl Parser {
    pub fn dec_gen(&mut self) -> Option<Generic> {
        let mut generic = IndexMap::new();

        if !self.might('<') {
            return Some(generic);
        }

        let idx = self.ensure_closed('>')?;

        loop {
            self.might('>');

            if self.idx == idx {
                break;
            }

            let key = self.identifier(true, false)?;
            let mut value = Vec::new();

            if self.might('>') {
                break;
            }

            if self.expect_char(&[':', ','])? != ':' {
                continue;
            }

            loop {
                value.push(self._trait()?);

                if self.might('>') || self.expect_char(&['+', ','])? != '+' {
                    break;
                }
            }

            generic.insert(key, value);
        }

        self.de.pop_front();

        Some(generic)
    }

    fn _trait(&mut self) -> Option<Trait> {
        let name = self.identifier(true, true)?;
        let mut sub = Vec::new();

        if !self.might('<') {
            return Some(Trait { name, sub });
        }

        let idx = self.ensure_closed('>')?;

        loop {
            self.might('>');

            if self.idx == idx {
                break;
            }

            sub.push(self._trait()?);

            if self.might('>') {
                break;
            }

            self.expect(&[','])?;
        }

        self.de.pop_front();

        Some(Trait { name, sub })
    }

    pub fn stm_gen(&mut self) -> Option<Option<Term>> {
        if self.peek() != '<' {
            return Some(None);
        }

        let idx = self.idx;
        let log = self.log.bypass();
        let mut is_gen = false;
        let mut buf = Vec::new();

        log.ignore = true;
        self.idx += 1;

        while let Some(typ) = self.typ() {
            let typ = self.span(typ);
            let tmp = self.might(',');

            is_gen |= tmp
                || typ.raw
                || typ.sub.len() != 0
                || typ.ptr != 0
                || typ.null != 0
                || !matches!(typ.kind.data, TypeKind::ID(_));
            buf.push(typ);

            if !tmp {
                break;
            }
        }

        log.ignore = false;
        let mut tmp = 0;
        let mut count = 1usize;

        while let Some(c) = self._next() {
            match c {
                ';' | '(' | ')' | '"' | '\'' => break,
                '>' => count -= 1,
                '<' => count += 1,
                _ if !c.is_ascii_whitespace() && tmp == 0 => tmp = self.idx,
                _ => {}
            }

            if count == 0 {
                is_gen = true;
                break;
            }
        }

        if !is_gen {
            self.idx = idx;
            return Some(None);
        }

        if count != 0 {
            log.err("unclosed generic parameter")?;
        }

        if tmp != 0 {
            log.rng.fill(tmp);
            let mut op = Vec::new();

            let tmp = match buf.last() {
                Some(Span {
                    data: Type { sub, .. },
                    ..
                }) => {
                    op.push(",");
                    if sub.is_empty() {
                        op.push("<");
                    }

                    "?"
                }
                _ => "<type>",
            };

            op.push(tmp);
            log.err_op(false, &op)?
        }

        log.rng = [idx + 1, self.idx];

        Some(Some(Term::Generic(buf)))
    }
}
