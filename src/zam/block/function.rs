use crate::{
    parser::span::Identifier,
    zam::typ::{Type, kind::TypeKind},
};

use super::{Hoistable, Parser};

impl Parser {
    pub fn fun(&mut self) -> Option<(Identifier, Hoistable)> {
        let name = self.identifier(true)?;
        let de = self.expect_char(&['<', '('])?;
        let gen = match de {
            '<' => self.gen()?,
            _ => Vec::new(),
        };

        if de == '<' {
            self.expect_char(&['(']);
        }

        let arg = self.fields(')')?;
        let ret = match self.skip_whitespace() {
            '{' => Type {
                kind: self.span(TypeKind::ID("()".into())),
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
            Hoistable::Function {
                arg,
                gen,
                ret,
                block: Some(self.block(false)?),
                public: false,
            },
        ))
    }
}
