use crate::zam::{typ::Type, Entity};

use super::{super::Parser, Statement};

impl Parser {
    pub fn var(&mut self, cte: bool) -> Option<Statement> {
        let id = self.identifier(true, false)?;
        let de = self.expect_char(&[':', '=', ';'])?;
        let typ = match de {
            ':' => self.typ()?,
            _ => Type::default(),
        };
        let mut exp = if de == '=' || de != ';' && self.expect_char(&['=', ';'])? == '=' {
            let tmp = self.exp([';'], true)?.0;

            self.expect_char(&[';'])?;

            tmp
        } else {
            Default::default()
        };

        exp.typ = typ;

        Some(Statement::Variable {
            id,
            data: Entity::Variable {
                exp,
                cte,
                done: false,
            },
        })
    }
}
