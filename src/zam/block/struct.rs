use indexmap::IndexMap;

use crate::parser::{
    span::{Identifier, ToSpan},
    Context,
};

use super::{Hoistable, Parser};

impl Parser {
    pub fn strukt(&mut self) -> Option<(Identifier, Hoistable)> {
        let name = self.identifier(true)?;
        let de = self.expect_char(&['<', '{'])?;
        let gen = match de {
            '<' => self.dec_gen()?,
            _ => IndexMap::new(),
        };

        if de == '<' {
            self.expect_char(&['{'])?;
        }

        self.ctx = Some(Context::Struct.span(name.rng));
        let fields = self.fields('}')?;
        self.ctx = None;

        Some((
            name,
            Hoistable::Struct {
                gen,
                fields,
                public: false,
            },
        ))
    }
}
