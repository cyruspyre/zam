use crate::parser::span::Identifier;

use super::{Hoistable, Parser};

impl Parser {
    pub fn strukt(&mut self) -> Option<(Identifier, Hoistable)> {
        let name = self.identifier(true)?;
        let rng = self.rng;
        let de = self.expect_char(&['<', '{'])?;
        let gen = match de {
            '<' => self.gen()?,
            _ => Vec::new(),
        };

        if de == '<' {
            self.expect_char(&['{']);
        }

        Some((
            name,
            Hoistable::Struct {
                gen,
                fields: self.fields('}')?,
                rng,
                public: false,
            },
        ))
    }
}
