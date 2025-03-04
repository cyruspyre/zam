use super::{super::Parser, Statement};

impl Parser {
    pub fn var(&mut self, cte: bool) -> Option<Statement> {
        let name = self.identifier(true)?;
        let typ = match self.expect_char(&[':', '=', ';'])? {
            ':' => Some(self.typ()?),
            _ => None,
        };
        let val = if typ.is_some() && self.expect_char(&['=', ';'])? == '=' {
            self.exp(';', true)?.0
        } else {
            Vec::new()
        };

        Some(Statement::Variable {
            name,
            typ,
            val,
            cte,
        })
    }
}
