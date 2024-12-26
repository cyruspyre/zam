use crate::source::Source;

use super::Statement;

impl Source {
    pub fn cond(&mut self) -> Statement {
        let mut cond = Vec::new();
        let mut default = None;

        loop {
            let exp = self.exp('{').0;

            if exp.is_empty() {
                self.err_op(true, &["<expression>"]);
            }

            let block = self.block(false);
            let mut tmp = self.idx;

            cond.push((exp, block));

            if self.word() == "else" {
                tmp = self.idx;

                if self.word() == "if" {
                    continue;
                }

                self.idx = tmp;

                default = Some(self.block(false))
            } else {
                self.idx = tmp
            }

            break;
        }

        Statement::Conditional { cond, default }
    }
}
