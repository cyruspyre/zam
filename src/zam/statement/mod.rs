mod r#loop;
mod variable;

use super::{Block, Entity, expression::Expression, identifier::Identifier};

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Variable { id: Identifier, data: Entity },
    Loop(Block),
    Expression(Expression),
    Break(Identifier),
    Continue(Identifier),
    Return(Expression),
}
