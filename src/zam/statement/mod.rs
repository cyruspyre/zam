mod conditional;
mod r#loop;
mod variable;

use super::{expression::Expression, typ::Type, Block};

#[derive(Debug, Clone)]
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
    Loop(Block),
    Block(Block),
    Expression(Expression),
    Break(String),
    Continue(String),
    Return(Expression),
}
