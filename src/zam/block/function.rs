use crate::{
    parser::misc::{Span, ToSpan},
    zam::typ::Type,
};

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
        let ret = match self.skip_whitespace() {
            '{' => Type {
                name: "()".into(),
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
            }
            .span(rng),
        ))
    }
}
