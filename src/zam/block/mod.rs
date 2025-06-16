mod function;
mod implement;
mod r#struct;
mod r#trait;

use std::{
    fmt::{Display, Formatter, Result},
    ops::Deref,
};

use indexmap::IndexMap;

use crate::{
    log::{Log, Point},
    misc::{Bypass, Ref},
    parser::span::ToSpan,
    zam::path::ZamPath,
};

use super::{
    expression::misc::Range, identifier::Identifier, statement::Statement, typ::generic::Generic,
    Entity, Parser,
};

#[rustfmt::skip]
pub type Impls = IndexMap<Ref<String>, IndexMap<Ref<ZamPath>, Vec<([Identifier; 2], Generic, Block)>>>;
type ImplsLocal = IndexMap<Ref<String>, Vec<([Identifier; 2], Generic, Block)>>;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Block {
    pub public: Vec<usize>,
    pub dec: IndexMap<Identifier, Entity>,
    pub stm: Vec<Statement>,
    pub global: bool,
    pub impls: ImplsLocal,
}

#[derive(PartialEq)]
pub enum BlockType {
    Impl,
    Local,
    Trait,
    Global,
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
        let mut flag = true;
        let mut dec: IndexMap<Identifier, _> = IndexMap::new();
        let mut impls = IndexMap::new();
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

            if tmp.is_empty() {
                break;
            }

            if tmp == "impl" {
                self.implement(&mut impls, global)?;
                continue;
            }

            'two: {
                let (mut k, v) = match tmp {
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
                    // "use" => ,
                    // "extern" => self.ext(&mut ext),
                    _ => break 'two,
                };

                if flag {
                    flag = false;
                    public.push(dec.len());
                }

                if let Some((prev, _)) = dec.get_key_value(&k) {
                    let rng = k.rng();

                    dup.entry(k)
                        .or_insert(vec![(prev.rng(), Point::Error, "first declared here")])
                        .push((rng, Point::Error, ""))
                } else {
                    let rng = k.rng();
                    let mut tmp = Vec::with_capacity(k.len() + 1);

                    for v in self.id.iter() {
                        tmp.push(v.deref().clone().span(rng));
                    }

                    tmp.push(k.pop().unwrap());
                    dec.insert(Identifier(tmp), v);
                }

                continue 'one;
            }

            if not_local {
                log.err(format!("expected keyword of {typ} context"))?
            }

            stm.push(match tmp {
                "let" | "cte" => self.var(tmp == "cte")?,
                "if" => self.cond()?,
                "for" | "loop" | "while" => self.r#loop(stm_ref, tmp)?,
                _ => {
                    self.idx = stamp;

                    let (exp, de) = self.exp([';'], false)?;
                    let used = de != '\0';

                    if used {
                        self._next();
                    }

                    // panic!("{de:?} {:?} {:?}", self.idx,  self.log.data[self.idx]);

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

        for (k, mut v) in dup {
            let msg = format!("identifier `{k}` declared multiple times");

            log(&mut v, Log::Error, &msg, "");
        }

        Some(Block {
            dec,
            stm,
            impls,
            global,
            public,
        })
    }
}
