use std::borrow::Cow;

use indexmap::IndexMap;

use crate::{
    log::Point,
    zam::{Entity, expression::misc::Range, identifier::Identifier},
};

use super::Parser;

impl Parser {
    pub fn structure(&mut self) -> Option<(Identifier, Entity)> {
        let name = self.identifier(true, false)?;
        let de = self.expect_char(&['<', '{'])?;
        let generic = match de {
            '<' => self.dec_gen()?,
            _ => IndexMap::new(),
        };

        if de == '<' {
            self.expect_char(&['{'])?;
        }

        let __ = self.log.ctx(
            name.rng(),
            Point::Error,
            Cow::Owned(format!("while parsing `{name}`")),
        );
        let fields = self.fields('}')?;

        Some((
            name,
            Entity::Struct {
                fields,
                generic,
                done: false,
                impls: IndexMap::new(),
                traits: IndexMap::new(),
            },
        ))
    }
}
