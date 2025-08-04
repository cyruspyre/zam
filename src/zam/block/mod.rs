mod function;
mod implement;
mod r#struct;
mod r#trait;
mod r#use;

use std::fmt::{Display, Formatter, Result};

use hashbrown::HashMap;
use indexmap::{IndexMap, map::Entry};

use crate::{
    log::{Log, Point},
    misc::{Bypass, Ref, RefMut},
    parser::span::Span,
    zam::path::ZamPath,
};

use super::{
    Entity, Parser, expression::misc::Range, identifier::Identifier, statement::Statement,
    typ::generic::Generic,
};

type Impl = Vec<([Identifier; 2], Generic, Block)>;
pub type Impls = HashMap<Ref<String>, IndexMap<Ref<ZamPath>, Impl>>;
type LocalImpls = HashMap<Ref<String>, Impl>;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Block {
    /// Indices of public declarations, where each index corresponds to `rng[0]`
    /// of the declaration's identifier.
    pub public: Vec<usize>,
    pub ext: IndexMap<Identifier, RefMut<Entity>>,
    pub dec: IndexMap<Identifier, Entity>,
    pub stm: Vec<Statement>,
    pub global: bool,
    pub impls: LocalImpls,
}

#[derive(PartialEq)]
pub enum BlockType {
    Impl,
    Local,
    Trait,
    Global,
}

impl Parser {
    pub fn block(&mut self, typ: BlockType) -> Option<Block> {
        self._block(typ, Vec::new())
    }

    pub fn _block(&mut self, typ: BlockType, mut stm: Vec<Statement>) -> Option<Block> {
        let idx = if typ != BlockType::Global {
            if self.log.data[self.idx] != '{' {
                self.expect(&['{'])?;
            }

            self.ensure_closed('}')?
        } else {
            0
        };

        let mut dup = IndexMap::new();
        let mut flag = false;
        let mut dec: IndexMap<Identifier, _> = IndexMap::new();
        let mut ext = IndexMap::new();
        let mut impls = HashMap::new();
        let mut public = Vec::new();
        let stm_ref = stm.bypass();
        let not_local = typ != BlockType::Local;
        let global = typ == BlockType::Global;
        let log = self.log.bypass();

        'one: loop {
            let stamp = self.idx;
            let Some(tmp) = self.id_or_symbol() else {
                break;
            };

            if self.idx == idx {
                self.de.pop_back();
                break;
            }

            let tmp = tmp.as_str();

            if match tmp {
                "" => break,
                "impl" => self.implement(&mut impls, global)?,
                "use" => self.r#use(&mut ext, &mut dup)?,
                _ => false,
            } {
                continue;
            }

            'two: {
                let (id, val) = match tmp {
                    "fn" => self.fun(typ != BlockType::Trait)?,
                    "pub" if not_local => {
                        flag = true;
                        continue 'one;
                    }
                    _ if typ == BlockType::Impl => break 'two,
                    "trait" => self.r#trait()?,
                    "struct" => self.structure()?,
                    "let" | "cte" if global => match self.var(tmp == "cte")? {
                        Statement::Variable { id, data } => (id, data),
                        _ => unreachable!(),
                    },
                    _ => break 'two,
                };

                if flag {
                    flag = false;
                    public.push(id[0].rng[0]);
                }

                insert(&mut dec, &mut dup, id, val);
                continue 'one;
            }

            if not_local {
                log.err(format!("expected keyword of {typ} context"))?
            }

            stm.push(match tmp {
                "let" | "cte" => self.var(tmp == "cte")?,
                "return" => Statement::Return({
                    let (exp, de) = self.exp([';'], false)?;

                    if de != '\0' {
                        self._next();
                    }

                    exp
                }),
                "for" | "loop" | "while" => self.r#loop(stm_ref, tmp)?,
                _ => {
                    self.idx = stamp;

                    let (exp, de) = self.exp([';'], false)?;
                    let used = de != '\0';

                    if used {
                        self._next();
                    }

                    if exp.data.is_empty() {
                        continue;
                    }

                    match used {
                        true => Statement::Expression(exp),
                        _ => Statement::Return(exp),
                    }
                }
            });
        }

        for v in ext.keys() {
            if let Some(og) = dec.get_key_value(v) {
                insert_dup(&mut dup, og.0.leaf_name(), v.rng());
            }
        }

        for (k, mut v) in dup {
            let msg = format!("identifier `{k}` defined multiple times");

            log(&mut v, Log::Error, &msg, "");
        }

        ext.sort_unstable_keys();

        Some(Block {
            ext,
            dec,
            stm,
            impls,
            global,
            public,
        })
    }
}

impl Block {
    pub fn id_is_public(&self, id: &Identifier) -> bool {
        self.public.contains(&id[0].rng[0])
    }
}

impl Display for BlockType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(match self {
            BlockType::Impl => "implementation",
            BlockType::Trait => "trait",
            BlockType::Local => "local",
            BlockType::Global => "global",
        })
    }
}

fn insert<V>(
    map: &mut IndexMap<Identifier, V>,
    dup: &mut IndexMap<&String, Vec<([usize; 2], Point, &'static str)>>,
    id: Identifier,
    val: V,
) {
    let rng = id.rng();
    let entry = map.entry(id);
    let Entry::Occupied(entry) = entry else {
        entry.insert_entry(val);
        return;
    };
    let key = entry.key().leaf_name();

    insert_dup(dup, key, rng)
}

fn insert_dup(
    map: &mut IndexMap<&String, Vec<([usize; 2], Point, &'static str)>>,
    dup: &Span<String>,
    og_rng: [usize; 2],
) {
    map.entry(unsafe { &*(&dup.data as *const _) })
        .or_insert_with(|| vec![(og_rng, Point::Error, "")])
        .push((dup.rng, Point::Error, ""))
}
