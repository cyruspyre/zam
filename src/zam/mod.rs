use std::path::PathBuf;

use block::Block;

use crate::parser::Parser;

pub mod block;
mod expression;
mod external;
mod fields;
mod statement;
mod typ;

pub struct Zam {
    pub parser: Parser,
    pub block: Block,
}

impl Zam {
    pub fn parse(path: PathBuf) -> Self {
        let mut parser = Parser::new(path);

        Self {
            block: parser.block(true),
            parser,
        }
    }
}
