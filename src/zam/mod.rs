use std::{fs::read_to_string, path::PathBuf};

use block::{Block, BlockType};
use expression::Expression;
use fields::Fields;
use identifier::Identifier;
use indexmap::IndexMap;
use typ::{generic::Generic, Type};

use crate::parser::{log::Log, Parser};

pub mod block;
pub mod expression;
mod external;
pub mod fields;
pub mod identifier;
pub mod statement;
pub mod typ;

#[derive(Default)]
pub struct Zam {
    pub parser: Parser,
    pub block: Block,
    pub mods: IndexMap<String, Zam>,
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
    pub fn parse(path: PathBuf, required: bool) -> Self {
        let res = read_to_string(&path);
        let err = res.is_err();
        let mut parser = Parser {
            data: res.unwrap_or_default().chars().collect(),
            path,
            idx: usize::MAX,
            ..Default::default()
        };

        if err && required {
            let path = &parser.path;
            let msg = format!(
                "couldn't find `{}` in `{}`",
                path.file_name().unwrap().display(),
                path.parent().unwrap().display()
            );

            parser.log::<&str, _, _>(&mut [], Log::Error, msg, "");
        }

        Self {
            mods: IndexMap::new(),
            block: parser.block(BlockType::Global).unwrap_or_default(),
            parser,
        }
    }
}
