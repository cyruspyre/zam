use std::{collections::VecDeque, fs::read_to_string, path::PathBuf};

use block::{Block, BlockType};
use expression::Expression;
use fields::Fields;
use identifier::Identifier;
use indexmap::IndexMap;
use typ::{generic::Generic, Type};

use crate::{
    log::{Log, Logger},
    misc::{Ref, RefMut},
    parser::Parser,
    zam::block::Impls,
};

pub mod block;
pub mod expression;
mod external;
pub mod fields;
pub mod identifier;
pub mod statement;
pub mod typ;

#[derive(Default)]
pub struct Zam {
    pub log: Logger,
    pub block: Block,
    pub parent: RefMut<Zam>,
    pub mods: IndexMap<String, Zam>,
    pub lookup: Lookup,
}

#[derive(Default)]
pub struct Lookup {
    pub vars: IndexMap<Ref<Identifier>, RefMut<Entity>>,
    pub decs: Vec<RefMut<IndexMap<Identifier, Entity>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Entity {
    Function {
        arg: Fields<Type>,
        gen: Generic,
        ret: Type,
        done: bool,
        block: Option<Block>,
    },
    Struct {
        gen: Generic,
        done: bool,
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
    pub fn parse(path: PathBuf, required: bool, impls: &mut Impls) -> Self {
        let res = read_to_string(&path);
        let err = res.is_err();
        let mut parser = Parser {
            log: Logger {
                path,
                data: res.unwrap_or_default().chars().collect(),
                ..Default::default()
            },
            impls: RefMut(impls),
            idx: usize::MAX,
            de: VecDeque::new(),
        };

        if err && required {
            let path = &parser.log.path;
            let msg = format!(
                "couldn't find `{}` in `{}`",
                path.file_name().unwrap().display(),
                path.parent().unwrap().display()
            );

            parser.log.call::<&str, _, _>(&mut [], Log::Error, msg, "");
        }

        let block = parser.block(BlockType::Global).unwrap_or_default();
        let mut log = parser.log;

        log.eof = true;

        Self {
            log,
            block,
            ..Default::default()
        }
    }
}
