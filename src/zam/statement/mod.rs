mod conditional;
mod r#loop;
mod variable;

use super::{Block, Entity, expression::Expression, identifier::Identifier};

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
