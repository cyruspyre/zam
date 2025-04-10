use std::{env::current_dir, path::PathBuf};

use block::Block;

use crate::parser::Parser;

pub mod block;
pub mod expression;
mod external;
mod fields;
pub mod statement;
pub mod typ;

pub struct Zam {
    pub parser: Parser,
    pub block: Block,
}

impl Zam {
    pub fn parse(path: PathBuf) -> Option<Self> {
        let mut parser = Parser::new(
            path.strip_prefix(current_dir().unwrap())
                .unwrap()
                .to_path_buf(),
        )?;

        Some(Self {
            block: parser.block(true)?,
            parser,
        })
    }
}
