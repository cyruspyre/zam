mod conditional;
mod variable;

use crate::{block::Block, expression::Expression, typ::Type};

#[derive(Debug)]
pub enum Statement {
    Variable {
        name: String,
        typ: Option<Type>,
        val: Expression,
        cte: bool,
    },
    Conditional {
        cond: Vec<(Expression, Block)>,
        default: Option<Block>,
    },
    Expression(Expression),
    Return(Expression),
}
