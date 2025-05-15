mod conditional;
mod r#loop;
mod variable;

use crate::parser::span::Identifier;

use super::{expression::Expression, Block, Entity};

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Variable {
        id: Identifier,
        data: Entity,
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
