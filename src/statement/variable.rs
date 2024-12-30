use crate::source::Source;

use super::Statement;

impl Source {
    pub fn var(&mut self, cte: bool) -> Statement {
        let name = self.identifier(false);
        let typ = match self.expect_char(&[':', '=', ';']) {
            ':' => Some(self.typ()),
            _ => None,
        };
        let val = if typ.is_some() && self.expect_char(&['=', ';']) == '=' {
            self.exp(';', true).0
        } else {
            Vec::new()
        };

        Statement::Variable {
            name,
            typ,
            val,
            cte,
        }
    }
}
