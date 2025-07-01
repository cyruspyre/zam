use indexmap::IndexMap;

use crate::{
    misc::Bypass,
    zam::{Entity, identifier::Identifier},
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

        let ctx = self.log.ctx.bypass();
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
