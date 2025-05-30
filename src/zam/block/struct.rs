use indexmap::IndexMap;

use crate::{
    parser::{span::ToSpan, Context},
    zam::{expression::misc::Range, identifier::Identifier, Entity},
};

use super::Parser;

impl Parser {
    pub fn structure(&mut self) -> Option<(Identifier, Entity)> {
        let name = self.identifier(true, false)?;
        let de = self.expect_char(&['<', '{'])?;
        let gen = match de {
            '<' => self.dec_gen()?,
            _ => IndexMap::new(),
        };

        if de == '<' {
            self.expect_char(&['{'])?;
        }

        self.ctx = Some(Context::Struct.span(name.rng()));
        let fields = self.fields('}')?;
        self.ctx = None;

        Some((
            name,
            Entity::Struct {
                gen,
                fields,
                done: false,
                impls: IndexMap::new(),
                traits: IndexMap::new(),
            },
        ))
    }
}
