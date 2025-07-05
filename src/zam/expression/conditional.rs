use crate::{
    parser::Parser,
    zam::{block::BlockType, expression::term::Term, typ::kind::TypeKind},
};

impl Parser {
    pub fn conditional(&mut self) -> Option<Term> {
        let mut conds = Vec::new();
        let mut default = None;

        loop {
            let mut exp = self.exp(['{'], true)?.0;

            exp.typ.kind.data = TypeKind::Bool;

            let block = self.block(BlockType::Local)?;

            conds.push((exp, block));

            if self.next_if(&["else"]).is_ok() {
                if self.next_if(&["if"]).is_ok() {
                    continue;
                }

                default = Some(self.block(BlockType::Local)?)
            }

            break;
        }

        Some(Term::Conditional { conds, default })
    }
}
