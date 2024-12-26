use crate::source::Source;

use super::Statement;

impl Source {
    pub fn var(&mut self, cte: bool) -> Statement {
        let name = self.identifier(false);
        let typ = match self.expect_char(&[':', '=']) {
            ':' => Some(self.typ()),
            _ => None,
        };

        if typ.is_some() {
            self.expect_char(&['=']);
        }

        let (val, end) = self.exp(';');

        if val.is_empty() {
            self.err_op(true, &["<expression>"]);
        }

        Statement::Variable {
            name,
            typ,
            val,
            cte,
        }
    }
}
