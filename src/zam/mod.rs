use std::{collections::VecDeque, fmt::Debug, fs::read_to_string, path::PathBuf};

use block::{Block, BlockType};
use expression::Expression;
use fields::Fields;
use hashbrown::HashMap;
use identifier::Identifier;
use indexmap::IndexMap;
use typ::{Type, generic::Generic};

use crate::{
    log::{Log, Logger},
    misc::{Ref, RefMut},
    parser::Parser,
    zam::{block::Impls, path::ZamPath},
};

pub mod block;
pub mod expression;
pub mod fields;
pub mod identifier;
mod misc;
pub mod path;
pub mod statement;
pub mod typ;

#[derive(Default)]
pub struct Zam {
    pub id: ZamPath,
    pub log: Logger,
    pub block: Block,
    pub parent: RefMut<Zam>,
    pub mods: HashMap<String, Zam>,
    pub lookup: Lookup,
}

#[derive(Default)]
pub struct Lookup {
    pub vars: IndexMap<Ref<Identifier>, RefMut<Entity>>,
    pub decs: Vec<(Ref<usize>, RefMut<IndexMap<Identifier, Entity>>)>,
    pub stamp: (Ref<ZamPath>, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Entity {
    Function {
        arg: Fields<Type>,
        generic: Generic,
        ret: Type,
        done: bool,
        block: Option<Block>,
    },
    Struct {
        generic: Generic,
        done: bool,
        fields: Fields<Type>,
        impls: IndexMap<Identifier, Entity>,
        traits: IndexMap<Identifier, [usize; 2]>,
    },
    Trait {
        generic: Generic,
        /// Will always contain `Entity::Function` or `Entity::Type`
        item: IndexMap<Identifier, Entity>,
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
            Entity::Trait { .. } => "trait",
            Entity::Struct { .. } => "struct",
            Entity::Variable { .. } => "variable",
            Entity::Function { .. } => "function",
        }
    }
}

impl Zam {
    pub fn parse(path: PathBuf, required: bool, impls: &mut Impls, id: ZamPath) -> Self {
        let res = read_to_string(&path);
        let err = res.is_err();
        let mut parser = Parser {
            id: Ref(&id),
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
            id,
            log,
            block,
            ..Default::default()
        }
    }
}
