use crate::source::Source;

use super::Statement;

impl Source {
    pub fn cond(&mut self) -> Statement {
        let mut cond = Vec::new();
        let mut default = None;

        let exp = self.exp('{').0;

        if exp.is_empty() {
            self.err_op(true, &["<expression>"]);
        }

        let block = self.block(false);

        cond.push((exp, block));

        let stamp = self.idx;

        while self.until_whitespace() == "else" {
            let stamp = self.idx;
            let two = self.rng;

            if self.until_whitespace() == "if" {
                cond.push((self.exp('{').0, self.block(false)));
            } else {
                self.idx = stamp;
                self.rng = two;
                default = Some(self.block(false));
                break;
            }
        }

        if cond.len() == 1 && default.is_none() {
            self.idx = stamp
        }

        Statement::Conditional { cond, default }
    }
}
