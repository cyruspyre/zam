use crate::{misc::Bypass, parser::Parser, zam::typ::kind::TypeKind};

use super::term::Term;

impl Parser {
    pub fn array(&mut self) -> Option<Term> {
        let tmp = self.bypass();
        let mut next = || {
            tmp.idx += 1;
            tmp.log.rng.fill(tmp.idx);
        };

        next();
        self.ensure_closed(']')?;

        let (exp, de) = self.exp([',', ';', ']'], false)?;
        let mut val = Vec::new();
        let tmp = de != ']';

        if exp.data.is_empty() {
            if tmp {
                self.log.err("expected <expression> thereafter")?
            }
        } else {
            if tmp {
                next()
            }

            val.push(exp);
        }

        let len = if de == ';' {
            let mut tmp = self.exp([']'], true)?.0;

            tmp.typ.kind.data = TypeKind::Integer {
                bit: u32::MAX,
                sign: false,
            };

            Some(tmp)
        } else {
            None
        };

        loop {
            if self.might(']') {
                break;
            }

            let exp = self.exp([], true)?.0;

            val.push(exp);

            if self.might(']') {
                break;
            }

            self.expect(&[','])?;
        }

        self.de.pop_back();

        Some(Term::Array { val, len })
    }
}
