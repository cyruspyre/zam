use crate::zam::{block::BlockType, typ::kind::TypeKind};

use super::{super::Parser, Statement};

impl Parser {
    pub fn cond(&mut self) -> Option<Statement> {
        let mut cond = Vec::new();
        let mut default = None;

        loop {
            let mut exp = self.exp(['{'], true)?.0;

            exp.typ.kind.data = TypeKind::Bool;

            let block = self.block(BlockType::Local)?;
            let mut tmp = self.idx;

            cond.push((exp, block));

            if self.word() == "else" {
                tmp = self.idx;

                if self.word() == "if" {
                    continue;
                }

                self.idx = tmp;

                default = Some(self.block(BlockType::Local)?)
            } else {
                self.idx = tmp
            }

            break;
        }

        Some(Statement::Conditional { cond, default })
    }
}
