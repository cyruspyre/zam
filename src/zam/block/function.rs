use crate::parser::misc::{Span, ToSpan};

use super::{Hoistable, Parser};

impl Parser {
    pub fn fun(&mut self) -> Option<(String, Span<Hoistable>)> {
        let name = self.identifier(true)?;
        let rng = self.rng;
        let de = self.expect_char(&['<', '('])?;
        let gen = match de {
            '<' => self.gen()?,
            _ => Vec::new(),
        };

        if de == '<' {
            self.expect_char(&['(']);
        }

        let arg = self.fields(')')?;

        self.expect(&["->"]);

        Some((
            name,
            Hoistable::Function {
                arg,
                gen,
                ret: self.typ()?,
                block: Some(self.block(false)?),
            }
            .span(rng),
        ))
    }
}
