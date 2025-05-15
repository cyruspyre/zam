mod function;
mod r#struct;

use indexmap::IndexMap;

use crate::{
    misc::Bypass,
    parser::{
        log::{Log, Point},
        span::Identifier,
    },
};

use super::{statement::Statement, Entity, Parser};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Block {
    pub public: Vec<usize>,
    pub dec: IndexMap<Identifier, Entity>,
    pub stm: Vec<Statement>,
}

impl Parser {
    pub fn block(&mut self, global: bool) -> Option<Block> {
        self._block(global, Vec::new())
    }

    pub fn _block(&mut self, global: bool, mut stm: Vec<Statement>) -> Option<Block> {
        if !global {
            self.expect(&['{'])?;
            self.ensure_closed('}')?;
        }

        let mut dup = IndexMap::new();
        let mut flag = true;
        let mut dec: IndexMap<Identifier, _> = IndexMap::new();
        let mut public = Vec::new();
        let stm_ref = stm.bypass();
        let de = match self.de.back() {
            Some(n) => n - 1,
            _ => 0,
        };

        'one: loop {
            if self.idx == de {
                self.de.pop_back();
                self._next();
                break;
            }

            let stamp = self.idx;
            let tmp = self.until_whitespace();
            let tmp = tmp.as_str();

            if tmp.is_empty() {
                break;
            }

            'two: {
                let (k, v) = match tmp {
                    "fn" => self.fun()?,
                    "struct" => self.strukt()?,
                    "let" | "cte" if global => match self.var(tmp == "cte")? {
                        Statement::Variable { id, data } => (id, data),
                        _ => unreachable!(),
                    },
                    // "use" => ,
                    // "extern" => self.ext(&mut ext),
                    "pub" => {
                        flag = true;
                        continue 'one;
                    }
                    _ => break 'two,
                };

                if flag {
                    flag = false;
                    public.push(dec.len());
                }

                if let Some((prev, _)) = dec.get_key_value(&k) {
                    dup.entry(k.data)
                        .or_insert(vec![(prev.rng, Point::Error, "first declared here")])
                        .push((k.rng, Point::Error, ""))
                } else {
                    dec.insert(k, v);
                }

                continue 'one;
            }

            if global {
                self.err("expected keyword of global context")?
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
            self.log(
                &mut v,
                Log::Error,
                &format!("identifier `{k}` declared multiple times"),
                "",
            );
        }

        Some(Block { dec, stm, public })
    }
}
