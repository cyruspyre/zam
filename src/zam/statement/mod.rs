mod conditional;
mod r#loop;
mod variable;

use crate::parser::span::Identifier;

use super::{expression::Expression, Block};

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Variable {
        name: Identifier,
        exp: Expression,
        cte: bool,
    },
    Conditional {
        cond: Vec<(Expression, Block)>,
        default: Option<Block>,
    },
    Loop(Block),
    Expression(Expression),
    Break(Identifier),
    Continue(Identifier),
    Return(Expression),
}
