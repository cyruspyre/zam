use super::{super::Parser, Statement};

impl Parser {
    pub fn cond(&mut self) -> Option<Statement> {
        let mut cond = Vec::new();
        let mut default = None;

        loop {
            let exp = self.exp('{', true)?.0;

            let block = self.block(false)?;
            let mut tmp = self.idx;

            cond.push((exp, block));

            if self.word() == "else" {
                tmp = self.idx;

                if self.word() == "if" {
                    continue;
                }

                self.idx = tmp;

                default = Some(self.block(false)?)
            } else {
                self.idx = tmp
            }

            break;
        }

        Some(Statement::Conditional { cond, default })
    }
}
