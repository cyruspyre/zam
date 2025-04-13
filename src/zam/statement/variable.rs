use crate::zam::typ::Type;

use super::{super::Parser, Statement};

impl Parser {
    pub fn var(&mut self, cte: bool) -> Option<Statement> {
        let name = self.identifier(true)?;
        let de = self.expect_char(&[':', '=', ';'])?;
        let typ = match de {
            ':' => self.typ()?,
            _ => Type::default(),
        };
        let mut exp = if de == '=' || de != ';' && self.expect_char(&['=', ';'])? == '=' {
            let tmp = self.exp(';', true)?.0;

            self.expect_char(&[';'])?;

            tmp
        } else {
            Default::default()
        };

        exp.typ = typ;

        Some(Statement::Variable { name, exp, cte })
    }
}
