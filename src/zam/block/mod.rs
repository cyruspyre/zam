mod function;
mod implement;
mod r#struct;

use std::{
    fmt::{Display, Formatter, Result},
    path::Path,
};

use indexmap::IndexMap;

use crate::{
    log::{Log, Point},
    misc::{Bypass, Ref},
};

use super::{
    expression::misc::Range, identifier::Identifier, statement::Statement, typ::generic::Generic,
    Entity, Parser,
};

pub type Impls = IndexMap<Ref<String>, IndexMap<Ref<Path>, Vec<([Identifier; 2], Generic, Block)>>>;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Block {
    pub public: Vec<usize>,
    pub dec: IndexMap<Identifier, Entity>,
    pub stm: Vec<Statement>,
}

#[derive(PartialEq)]
pub enum BlockType {
    Impl,
    Local,
    Global,
}

impl Display for BlockType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(match self {
            BlockType::Impl => "implementation",
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
        if typ != BlockType::Global {
            if self.log.data[self.idx] != '{' {
                self.expect(&['{'])?;
            }

            self.ensure_closed('}')?;
        }

        let mut dup = IndexMap::new();
        let mut flag = true;
        let mut dec: IndexMap<Identifier, _> = IndexMap::new();
        let mut public = Vec::new();
        let stm_ref = stm.bypass();
        let log = self.log.bypass();
        let de = match self.de.back() {
            Some(n) => n - 1,
            _ => 0,
        };

        'one: loop {
            self.skip_whitespace();

            if self.idx == de {
                self.de.pop_back();
                self._next();
                break;
            }

            let not_local = typ != BlockType::Local;
            let stamp = self.idx;
            let Some(tmp) = self.id_or_symbol() else {
                break;
            };
            let tmp = tmp.as_str();

            if tmp.is_empty() {
                break;
            }

            if tmp == "impl" {
                self.implement()?;
                continue;
            }

            'two: {
                let (k, v) = match tmp {
                    "fn" => self.fun()?,
                    "pub" if not_local => {
                        flag = true;
                        continue 'one;
                    }
                    _ if typ == BlockType::Impl => break 'two,
                    "struct" => self.structure()?,
                    "let" | "cte" => match self.var(tmp == "cte")? {
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
                    dec.insert(k, v);
                }

                continue 'one;
            }

            if not_local {
                log.err(format!("expected keyword of {typ} context"))?
            } else if stamp == de {
                continue;
            }

            stm.push(match tmp {
                "let" | "cte" => self.var(tmp == "cte")?,
                "if" => self.cond()?,
                "for" | "loop" | "while" => self.r#loop(stm_ref, tmp)?,
                _ => {
                    self.idx = stamp;
                    // self.rng.fill(0); // safe to use this as flag for assignable expression

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

        for (k, mut v) in dup {
            log(
                &mut v,
                Log::Error,
                &format!("identifier `{k}` declared multiple times"),
                "",
            );
        }

        Some(Block { dec, stm, public })
    }
}
