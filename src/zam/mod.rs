use std::path::PathBuf;

use block::Block;

use crate::parser::Parser;

mod block;
mod expression;
mod external;
mod fields;
mod statement;
mod typ;

pub fn parse(path: PathBuf) -> Block {
    Parser::new(path).block(true)
}
