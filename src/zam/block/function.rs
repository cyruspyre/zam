use indexmap::IndexMap;

use crate::zam::{
    Entity,
    identifier::Identifier,
    typ::{Type, kind::TypeKind},
};

use super::{BlockType, Parser};

impl Parser {
    pub fn fun(&mut self, require_body: bool) -> Option<(Identifier, Entity)> {
        let name = self.identifier(true, false)?;
        let de = self.expect_char(&['<', '('])?;
        let generic = match de {
            '<' => self.dec_gen()?,
            _ => IndexMap::new(),
        };

        if de == '<' {
            self.expect_char(&['('])?;
        }

        let arg = self.fields(')')?;
        let ret = match self.skip_whitespace() {
            '{' => Type {
                kind: self.span(TypeKind::Unknown),
                sub: Vec::new(),
                ptr: 0,
                raw: false,
                null: 0,
            },
            _ => {
                self.expect(&["->"])?;
                self.typ()?
            }
        };

        Some((
            name,
            Entity::Function {
                arg,
                ret,
                generic,
                done: false,
                block: Some(self.block(BlockType::Local)?),
            },
        ))
    }
}
