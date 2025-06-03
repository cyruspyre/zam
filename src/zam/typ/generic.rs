use indexmap::IndexMap;

use crate::{
    misc::Bypass,
    parser::span::Span,
    zam::{
        expression::term::Term,
        identifier::Identifier,
        typ::{kind::TypeKind, Type},
    },
};

use super::{r#trait::Trait, Parser};

pub type Generic = IndexMap<Identifier, Vec<Trait>>;

impl Parser {
    pub fn dec_gen(&mut self) -> Option<Generic> {
        let mut data = IndexMap::new();
        let log = self.log.bypass();

        if self.might('>') {
            return Some(data);
        }

        'main: loop {
            let tmp = self.identifier(false, false)?;

            if tmp.is_empty() {
                log.err_op(false, &[">", "<identifier>"])?
            }

            let de = self.expect_char(&[':', '>'])?;
            data.insert(tmp, Vec::new());

            if de == '>' {
                break;
            }

            loop {
                // todo: try to eliminate trt() as it looks redundant and is used only once
                data.last_mut().unwrap().1.push(self.trt()?);

                match self.expect_char(&['+', ',', '>'])? {
                    '+' => {}
                    ',' => break,
                    _ => break 'main,
                }
            }
        }

        Some(data)
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
