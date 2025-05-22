use std::{env::current_dir, path::PathBuf};

use block::{Block, BlockType};
use expression::Expression;
use fields::Fields;
use indexmap::IndexMap;
use typ::{generic::Generic, Type};

use crate::parser::{span::Identifier, Parser};

pub mod block;
pub mod expression;
mod external;
pub mod fields;
pub mod statement;
pub mod typ;

pub struct Zam {
    pub parser: Parser,
    pub block: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Entity {
    Function {
        arg: Fields<Type>,
        gen: Generic,
        ret: Type,
        block: Option<Block>,
    },
    Struct {
        gen: Generic,
        fields: Fields<Type>,
        impls: IndexMap<Identifier, Entity>,
        traits: IndexMap<Identifier, [usize; 2]>,
    },
    Variable {
        exp: Expression,
        cte: bool,
        done: bool,
    },
}

impl Entity {
    pub fn name(&self) -> &str {
        match self {
            Entity::Struct { .. } => "struct",
            Entity::Variable { .. } => "variable",
            Entity::Function { .. } => "function",
        }
    }
}

impl Zam {
    pub fn parse(path: PathBuf) -> Option<Self> {
        let mut parser = Parser::new(
            path.strip_prefix(current_dir().unwrap())
                .unwrap()
                .to_path_buf(),
        )?;

        Some(Self {
            block: parser.block(BlockType::Global)?,
            parser,
        })
    }
}
